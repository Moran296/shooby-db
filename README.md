# shooby-db
light-weight configurations db for embedded rust projects

This crate is mostly a macro that creates a configuration table with only static memory and fast access for real time embedded devices.

For the moment it is still full of unsafe code and the type options are bool, u32, str and blob which is any sized repr[packed] struct.
I need to check about the repr packed though.

example:
```
// just a struct to use as blob...
struct WifiSettings {
  phy: PHY,
  something: u32,
};

shooby_db!(WIFI_CONFIG =>
    {SSID, String, "MY_HOUSE", 32, PERSISTENT},
    {PASSWORD, String, "12345678", 24, PERSISTENT},
    {AUTO_CONNECT, Bool, false, None, PERSISTENT},
    {CONNECTION_RETRIES, Int, 10, Some((0, 30)), NON_PERSISTENT},
    {OTHER_SETTINGS, Blob, WifiSettings {phy: PHY::BGN, something: 42} , std::mem::size_of::<WifiSettings>(), PERSISTENT},
);

fn main() {
     let mut db = WIFI_CONFIG::DB::take();
     db.init();
     
     {
         let reader = db.reader();
         println!("ssid: {}", reader[WIFI_CONFIG::ID::SSID].get_string());
     }
     
     db.write_with(|writer| {
            writer[WIFI_CONFIG::ID::SSID].set_string("something else");
     });

}

```

Please note that this is the start, the work is in progress and API will change!
