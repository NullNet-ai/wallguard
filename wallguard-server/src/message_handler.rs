use crate::{datastore::DatastoreWrapper, proto::wallguard::Packets};

pub async fn worker_task(
    rx: async_channel::Receiver<Packets>,
    mut datastore: Option<DatastoreWrapper>,
) {
    loop {
        let message = match rx.recv().await {
            Ok(message) => message,
            Err(e) => {
                println!("Receiver error: {}. Task Id {:?}", e, tokio::task::id());
                continue;
            }
        };

        // Do I need to clone here?
        // Did it to satisfy the borrow checker
        let Some(authentication) = message.auth.clone() else {
            eprintln!("Unauthenticated message. Skipping data transmission...");
            continue;
        };

        println!("Received {} packets", message.packets.len());

        let parsed_message = crate::parser::msg_parser::parse_message(message);
        if parsed_message.records.is_empty() {
            eprintln!("No valid packets in the message. Skipping...");
            continue;
        };

        let Some(datastore) = datastore.as_mut() else {
            eprintln!("Datastore functionality is disabled. Skipping data transmission...");
            continue;
        };

        match datastore
            .packets_insert(authentication.token, parsed_message)
            .await
        {
            Ok(response) if !response.success => {
                let error = response.error;
                let message = response.message;
                eprintln!("Error saving a message: {error} {message}");
            }
            Ok(_) => {}
            Err(status) => {
                eprintln!("Error saving a message: {status}");
            }
        }
    }
}
