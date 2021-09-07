use std::{thread, time::Duration};

#[tokio::main]
#[test]
async fn tokio_main() {
    let aux = comn::autex::Autex::new();
    let f1 = async {
        let g = aux.lock().await;
        println!(
            "1, {:?}\t\t get lock and sleep for 5 seconds",
            thread::current().id()
        );
        tokio::time::sleep(Duration::from_secs(5)).await;
        drop(g);
        print!("1, {:?}\t\t drop lock and quit\n", thread::current().id());
    };

    let f2 = async {
        print!("2, {:?}\t\t trying get lock ...\n", thread::current().id());
        let g = aux.lock().await;
        print!("2, {:?}\t\t get lock\n", thread::current().id());
    };

    let f3 = async {
        println!("yield to f3");
    };
    tokio::join!(f1, f2, f3);
}
