#[tokio::test]
async fn test() {
    let file = crate::File2Dl::new("https://dl.google.com/tag/s/appguid%3D%7B8A69D345-D564-463C-AFF1-A69D9E530F96%7D%26iid%3D%7B96CE5827-B46C-0844-268E-C65B80933F1E%7D%26lang%3Den-GB%26browser%3D3%26usagestats%3D0%26appname%3DGoogle%2520Chrome%26needsadmin%3Dprefers%26ap%3Dx64-statsdef_1%26installdataindex%3Dempty/chrome/install/ChromeStandaloneSetup64.exe", "Downloads").await.unwrap();
    file.single_thread_dl().await.unwrap();
}
