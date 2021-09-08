use std::{thread, time::Duration};
extern crate comn;

#[tokio::main]
#[test]
async fn tokio_main() {
    let aux0 = std::sync::Arc::new(comn::autex::Autex::new());

    let aux = aux0.clone();
    let f1 = async move {
        let g = aux.lock().await;
        println!(
            "1, {:?}\t\t hold lock for 5 seconds",
            thread::current().id()
        );
        tokio::time::sleep(Duration::from_secs(5)).await;
        println!("1, {:?}\t\t unlock and quit", thread::current().id());
        drop(g);
    };

    let aux = aux0.clone();
    let f2 = async move {
        println!("2, {:?}\t\t getting lock ...", thread::current().id());
        let g = aux.lock().await;
        println!("2, {:?}\t\t got lock", thread::current().id());
        drop(g);
    };

    let aux = aux0.clone();
    let f3 = async move {
        println!("3, {:?}\t\t getting lock ...", thread::current().id());
        let g = aux.lock().await;
        println!("3, {:?}\t\t got lock", thread::current().id());
        drop(g);
    };
    drop(aux0);

    let h1 = tokio::spawn(f1);
    let h2 = tokio::spawn(f2);
    let h3 = tokio::spawn(f3);
    tokio::join!(h1, h2, h3);
}
