enum Message {
    SetThing { value: i32 }, 
    GetThing { tx: oneshot::Sender<i32> }
}

