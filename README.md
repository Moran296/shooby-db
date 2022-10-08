# shooby-db
light-weight configurations db for embedded rust projects

This crate is mostly a macro that creates a configuration table with only static memory and fast access for real time embedded devices.

It will support persistency, observing, thread safety.
No heap allocation is used.

For the moment it is still full of unsafe code and the type options are bool, u32, str and blob which is any sized repr[packed] struct.
I need to check about the repr packed though.

### example:
```
// just a struct to use as blob...
struct WifiSettings {
  phy: PHY,
  something: u32,
};

const WIFI_SETTINGS_SIZE: USIZE = std::mem::size_of::<WifiSettings>();

shooby_db!(WIFI_CONFIG =>
   //NAME                TYPE      DEFAULT       LIMITS/SIZE/RANGES                                    PERSISTENCY
   
    {SSID,               String,   "MY_HOUSE",   32,                                                   PERSISTENT},
    {PASSWORD,           String,   "12345678",   24,                                                   PERSISTENT},
    {AUTO_CONNECT,       Bool,     false,        None,                                                 PERSISTENT},
    {CONNECTION_RETRIES, Int,      10,           Some((0, 30)),                                        NON_PERSISTENT},
    {OTHER_SETTINGS,     Blob,     WifiSettings  {phy: PHY::BGN, something: 42} , WIFI_SETTINGS_SIZE,  PERSISTENT},
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
This created the next table
| NAME | TYPE | DEFAULT | LIMITS/SIZE/RANGES | PERSISTENCY |
| --- | --- | --- | --- | --- |
| SSID | String | "MY_HOUSE" | 32 Bytes max | PERSISTENT |
| PASSWORD | String | "12345678" | 24 Bytes max | PERSISTENT |
| AUTO_CONNECT | Bool | false | None | PERSISTENT |
| CONNECTION_RETRIES | Int | 10 | minimum: 0, maximum: 30 | NON_PERSISTENT |
| OTHER_SETTINGS | Blob | phy: PHY::BGN, something: 42 | size of struct only | PERSISTENT |


Please note that this is the start, the work is in progress and API will change!

### PLANS
  - [ ] add persistency trait
  - [ ] add subscriber/observer trait
  - [ ] call persistency and observers upon writes
  - [ ] factory reset (including in persistency)
  - [ ] add more types
  - [ ] add thread safety
  - [ ] cut on unsafe
  - [ ] test alignment, packed, UB
  - [ ] test and use in esp32/stm32
  - [ ] benchmark speed and size
