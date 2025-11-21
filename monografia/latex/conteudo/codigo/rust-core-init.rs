    impl Core {
        // ...
        pub async fn init(mut self, mut rx: mpsc::Receiver<Message>) {
            while let Some(message) = rx.recv().await {
                match message {
                    Message::SetThing { value } => {
                        self.handle_set_thing(value);
                    }
                    Message::GetThing { tx } => {
                        let result = self.handle_get_thing();
                        let _ = tx.send(result);
                    }
                }
            }
        }
        
        fn handle_set_thing(&mut self, value: i32) {
            self.thing = value;
        }
        
        fn handle_get_thing(&self) -> i32 {
            self.thing
        }
    }

