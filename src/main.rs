use jsonparser::JsonValue::*;
// use std::collections::HashMap;

fn main() {
    let jnull = JsonNull;
    println!("JsonNull: {:?}", jnull);
    let jbool = JsonBool(true);
    println!("JsonBool: {:?}", jbool);
    let jnumber = JsonNumber(1234);
    println!("JsonNumber: {:?}", jnumber);
    let jstring = JsonString("hello".into());
    println!("JsonString: {:?}", jstring);
    // let jarray = JsonArray(Box::new(vec![jnull, jbool, jnumber]));
    // println!("JsonArray: {:?}", jarray);

    // let jobject = JsonObject(Box::new(HashMap::from([
    //     ("hello".to_owned(), JsonString("world".to_owned())),
    //     ("number".to_owned(), JsonNumber(1234)),
    // ])));
    // println!("JsonObject: {:?}", jobject);
}
