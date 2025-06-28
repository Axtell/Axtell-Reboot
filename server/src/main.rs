use axtell_server::api::{Query, Schema};
use juniper::{EmptyMutation, EmptySubscription};

fn main() {
    let schema = Schema::new(Query, EmptyMutation::new(), EmptySubscription::new());
    println!("{}", schema.as_sdl());
}
