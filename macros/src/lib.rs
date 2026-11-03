//! `hnu_query` 可观测性相关过程宏。
use proc_macro::TokenStream;
use quote::quote;
use syn::{
    Ident, ItemFn, LitStr, Token,
    parse::{Parse, ParseStream},
    parse_macro_input,
    punctuated::Punctuated,
};

/// 为 async 函数附加 tracing span，并注入 fetch/parse 阶段计时的 task-local 作用域。
///
/// # Examples
///
/// ```ignore
/// use crate::utils::obs::{fetch_time, parse_time, traced};
///
/// #[traced(subsystem = "ai", skip(token))]
/// pub async fn get_token_list(token: &AiToken) -> Result<...> {
///     let json_str = fetch_time!(fetch::token_list(token).await)?;
///     parse_time!(parse::token_list(&json_str))
/// }
/// ```
///
/// 宏会：
/// 1. 生成 `#[instrument]`，并自动为 `fetch_ms` / `parse_ms` 加上 `Empty` 占位（单位：毫秒）
/// 2. 用 task-local 计时器包住函数体；函数结束（含 `?` 失败）时把累计耗时写入上述字段
/// 3. 函数体内用 [`fetch_time!`] / [`parse_time!`] 把各阶段耗时累加到该计时器
///
/// # 参数
///
/// - `subsystem = "..."`：必填
/// - `skip(...)`：可选，敏感参数
/// - `record(...)`：可选，额外的 `Empty` 占位字段（`fetch_ms` / `parse_ms` 已自动包含）
/// - `err` / `no_err`：默认 `err`
/// - `level = "..."`：可选
#[proc_macro_attribute]
pub fn traced(attr: TokenStream, item: TokenStream) -> TokenStream {
    let args = parse_macro_input!(attr as TracedArgs);
    let mut input = parse_macro_input!(item as ItemFn);

    if input.sig.asyncness.is_none() {
        return syn::Error::new_spanned(
            input.sig.fn_token,
            "#[traced] only supports async functions (needs task-local scope)",
        )
        .to_compile_error()
        .into();
    }

    let subsystem = args.subsystem;
    let skip = args.skip;
    let record_fields = args.record_fields;
    let level = args.level;
    let err = args.err;

    let skip_tokens = if skip.is_empty() {
        quote! {}
    } else {
        quote! { skip(#(#skip),*), }
    };

    let level_tokens = if let Some(level) = level {
        quote! { level = #level, }
    } else {
        quote! {}
    };

    let err_tokens = if err {
        quote! { err }
    } else {
        quote! {}
    };

    let extra_empty_fields = record_fields.iter().map(|name| {
        quote! { #name = ::tracing::field::Empty, }
    });

    let stmts = &input.block.stmts;
    let new_block = syn::parse2(quote! {{
        crate::utils::obs::with_phase_timers(async {
            let _obs_flush = crate::utils::obs::FlushPhaseTimersOnDrop;
            #(#stmts)*
        }).await
    }});
    let new_block = match new_block {
        Ok(block) => block,
        Err(e) => return e.to_compile_error().into(),
    };
    input.block = new_block;

    let expanded = quote! {
        #[cfg_attr(
            feature = "tracing",
            ::tracing::instrument(
                #level_tokens
                #skip_tokens
                fields(
                    subsystem = #subsystem,
                    fetch_ms = ::tracing::field::Empty,
                    parse_ms = ::tracing::field::Empty,
                    #(#extra_empty_fields)*
                ),
                #err_tokens
            )
        )]
        #input
    };

    expanded.into()
}

struct TracedArgs {
    subsystem: LitStr,
    skip: Vec<Ident>,
    record_fields: Vec<Ident>,
    level: Option<LitStr>,
    err: bool,
}

impl Parse for TracedArgs {
    fn parse(input: ParseStream<'_>) -> syn::Result<Self> {
        let mut subsystem = None;
        let mut skip = Vec::new();
        let mut record_fields = Vec::new();
        let mut level = None;
        let mut err = true;

        while !input.is_empty() {
            let ident: Ident = input.parse()?;
            match ident.to_string().as_str() {
                "subsystem" => {
                    input.parse::<Token![=]>()?;
                    subsystem = Some(input.parse()?);
                }
                "level" => {
                    input.parse::<Token![=]>()?;
                    level = Some(input.parse()?);
                }
                "skip" => {
                    let content;
                    syn::parenthesized!(content in input);
                    skip = Punctuated::<Ident, Token![,]>::parse_terminated(&content)?
                        .into_iter()
                        .collect();
                }
                "record" => {
                    let content;
                    syn::parenthesized!(content in input);
                    record_fields = Punctuated::<Ident, Token![,]>::parse_terminated(&content)?
                        .into_iter()
                        .collect();
                }
                "err" => {
                    err = true;
                }
                "no_err" => {
                    err = false;
                }
                other => {
                    return Err(syn::Error::new(
                        ident.span(),
                        format!(
                            "unknown traced argument `{other}`, expected subsystem/skip/record/level/err/no_err"
                        ),
                    ));
                }
            }

            if input.peek(Token![,]) {
                input.parse::<Token![,]>()?;
            }
        }

        let subsystem = subsystem.ok_or_else(|| {
            syn::Error::new(
                proc_macro2::Span::call_site(),
                "traced requires `subsystem = \"...\"`",
            )
        })?;

        Ok(Self {
            subsystem,
            skip,
            record_fields,
            level,
            err,
        })
    }
}
