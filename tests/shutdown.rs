mod test_utils;
use async_std::task;
use std::time::Duration;

#[async_std::test]
async fn shutdown_waits_for_clients() {
    let port = test_utils::find_port().await;
    let mut app = tide::new();
    app.at("/").get(|_| async move {
        task::sleep(Duration::from_millis(500)).await;
        Ok("Yeet")
    });

    let server_clone = app.clone();
    task::spawn(async move {
        app.listen(("localhost", port)).await?;
        Result::<(), http_types::Error>::Ok(())
    });

    let clients: Vec<_> = (0..5).map(|_| task::spawn(async move {
        task::sleep(Duration::from_millis(100)).await;
        let mut res = surf::get(format!("http://localhost:{}", port))
            .await
            .unwrap();
        assert_eq!(res.status(), 200);
        let string = res.body_string().await.unwrap();
        assert_eq!(string, "Yeet");
    })).collect();

    task::sleep(Duration::from_millis(200)).await;
    server_clone.shutdown();

    for client in clients {
        client.await;
    }
}

#[async_std::test]
async fn shutdown_responds_with_503_after() {
    let port = test_utils::find_port().await;
    let mut app = tide::new();
    app.at("/").get(|_| async move {
        task::sleep(Duration::from_millis(500)).await;
        Ok("Yeet")
    });

    let server_clone = app.clone();
    task::spawn(async move {
        app.listen(("localhost", port)).await?;
        Result::<(), http_types::Error>::Ok(())
    });

    task::spawn(async move {
        task::sleep(Duration::from_millis(100)).await;
        let mut res = surf::get(format!("http://localhost:{}", port))
            .await
            .unwrap();
        assert_eq!(res.status(), 200);
        let string = res.body_string().await.unwrap();
        assert_eq!(string, "Yeet");
    });

    task::sleep(Duration::from_millis(200)).await;
    server_clone.shutdown();

    let client = task::spawn(async move {
        task::sleep(Duration::from_millis(100)).await;
        let res = surf::get(format!("http://localhost:{}", port))
            .await
            .unwrap();
        assert_eq!(res.status(), 503);
    });
    client.await;
}