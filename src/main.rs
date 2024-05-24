use regex::Regex;
use tokio::io::AsyncWriteExt;
use tokio::task::JoinSet;

#[tokio::main]
async fn main() {
    let pattern = r#"<a href="([^"]+)">"#;
    let re = Regex::new(pattern).unwrap();
    let url = "https://www.windhoekcc.org.na/documents/";
    let client = reqwest::Client::new();
    download(url, &client, &re).await;
    println!("Downloaded all files");
}

async fn download(url: &'static str, client: &reqwest::Client, re: &Regex) {
    let response = client.get(url).send().await.unwrap();
    let body = response.text().await.unwrap();

    let files = re
        .captures_iter(&body)
        .filter_map(|f| f.get(1).map(|f| f.as_str().to_string()))
        .collect::<Vec<String>>();

    let mut tasks = JoinSet::new();

    for file in files {
        if !file.ends_with(".pdf") {
            continue;
        }

        let client = client.clone();
        tasks.spawn(async move {
            let file = file.split("/").last().unwrap();
            let url = format!("{}{}", url, file);
            let file = if file.contains("form") {
                format!("./assets/forms/{}", file)
            } else {
                format!("./assets/{}", file)
            };
            let mut file = tokio::fs::File::create(file).await.unwrap();
            println!("Downloading: {}", url);
            let mut response = client.get(&url).send().await.unwrap();
            println!("Downlaoding: {}", url);
            while let Some(chunk) = response.chunk().await.unwrap() {
                file.write_all(&chunk).await.unwrap();
            }
        });
    }

    while let Some(res) = tasks.join_next().await {
        println!("Download result: {res:?}");
    }
}
