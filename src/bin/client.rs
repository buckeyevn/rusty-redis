use redis::Commands;

fn main() {
    println!("client");
    // connect to redis
    let client = redis::Client::open("redis://127.0.0.1/").unwrap();
    let mut con = client.get_connection().unwrap();
    // throw away the result, just make sure it does not fail
    let _: () = con.set("my_key", 42).unwrap();
    // read back the key and return it.  Because the return value
    // from the function is a result for integer this will automatically
    // convert into one.
    let foo: redis::RedisResult<isize> = con.get("my_key");
    println!("{:?}", foo)
}
