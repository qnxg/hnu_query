use dotenvy::dotenv_iter;

fn main() {
    load_dotenv();
}

/// 加载.env文件到环境变量
fn load_dotenv() {
    let Ok(dotenv_items) = dotenv_iter() else {
        return;
    };
    for item in dotenv_items {
        let Ok((k, v)) = item else { continue };
        println!("cargo:rustc-env={k}={v}")
    }
}
