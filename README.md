## Notes
 - ReadInputRegisters from tokio_modbus expects u16 vec arg to be send by server
   - workarround applied for negative values (i16): 1st u16 from vec is a negative indicator, 2nd u16 is the value
   - workarround applied for bigger values (u32): Split the value into 2 u16 registers as cannot be send as hex

# Server with fake data

cargo run --bin modbus-server


# Client

cargo run --bin modbus-client


# Example

## Server

![image](https://github.com/user-attachments/assets/2ce1bf99-257a-4ecd-8159-399ba6ca4a6c)


## Client

![image](https://github.com/user-attachments/assets/7a1f96a1-8618-4cf5-a248-349958c38533)


