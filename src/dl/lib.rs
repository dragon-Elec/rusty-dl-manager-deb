#[tokio::test]
async fn test() {
    let file =
        crate::dl::file2dl::File2Dl::new("https://ash-speed.hetzner.com/10GB.bin", "Downloads")
            .await
            .unwrap();
    // file.single_thread_dl().await.unwrap();
}
