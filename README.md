## Notes
 - ReadInputRegisters from tokio_modbus expects Vec<u16> args to be send by server
   - workarround applied for negative (i16): 1st u16 from vec is a negative indicator, 2nd u16 is the value
   - workarround applied bigger (u32): Split the value into 2 u16 registers

# Server with fake data

cargo run --bin modbus-server


# Client

cargo run --bin modbus-client


# Example

## Server

![image](https://github.com/user-attachments/assets/ac3f4728-2a70-41fd-9d45-b073de0e7ee0)


## Client

![image](https://github.com/user-attachments/assets/3b610cef-84f7-407e-9a51-0d364188d837)

