fn main() {
    slint_build::compile("ui/app-window.slint").expect("Slint build failed");

    // Windows: 添加图标和应用程序信息
    #[cfg(windows)]
    {
        let mut res = winresource::WindowsResource::new();
        res.set_icon("icon.ico")
            .set("ProductName", "StegSolve-RS")
            .set("FileDescription", "Steganography Analysis Tool")
            .set("LegalCopyright", "Copyright 2026");

        // 如果没有ico文件，忽略图标错误
        let _ = res.compile();
    }
}
