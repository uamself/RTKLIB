use std::fs::File;
use std::io::{self, Read};

use rtcm2rinex::rtcm::{RtcmContext, RtcmMessageType};

fn main() -> io::Result<()> {
    println!("Debug RTCM parser - processing continuous data stream");
    
    let mut file = File::open("A053900482.txt")?;
    let mut buffer = Vec::new();
    file.read_to_end(&mut buffer)?;
    
    println!("File size: {} bytes", buffer.len());
    
    let mut context = RtcmContext::new().expect("Failed to create RTCM context");
    let mut message_count = 0;
    let mut bytes_processed = 0;
    let mut rtcm3_count = 0;
    let mut rtcm2_count = 0;
    let mut error_count = 0;
    
    // Process each byte in the data stream
    for (i, &byte) in buffer.iter().enumerate() {
        bytes_processed = i + 1;
        
        match context.process_byte(byte) {
            Ok(Some(message)) => {
                message_count += 1;
                match message {
                    RtcmMessageType::Rtcm3(msg_type) => {
                        rtcm3_count += 1;
                        if message_count <= 10 {  // Show details for first 10 messages
                            println!("Message #{}: RTCM3 type {:?} at byte {}", 
                                   message_count, msg_type, i+1);
                        }
                    },
                    RtcmMessageType::Rtcm2(msg_type) => {
                        rtcm2_count += 1;
                        if message_count <= 10 {
                            println!("Message #{}: RTCM2 type {:?} at byte {}", 
                                   message_count, msg_type, i+1);
                        }
                    },
                    RtcmMessageType::Unknown(msg_type) => {
                        if message_count <= 10 {
                            println!("Message #{}: Unknown type {} at byte {}", 
                                   message_count, msg_type, i+1);
                        }
                    }
                }
                
                // Show progress every 100 messages
                if message_count % 100 == 0 {
                    println!("Processed {} messages...", message_count);
                }
            },
            Ok(None) => {
                // Continue processing, no complete message yet
            },
            Err(e) => {
                error_count += 1;
                if error_count <= 10 {  // Show first 10 errors
                    println!("Error at byte {}: {:?}", i+1, e);
                }
            }
        }
    }
    
    println!("\n=== Processing Summary ===");
    println!("Total bytes processed: {}", bytes_processed);
    println!("Total messages found: {}", message_count);
    println!("RTCM3 messages: {}", rtcm3_count);
    println!("RTCM2 messages: {}", rtcm2_count);
    println!("Errors encountered: {}", error_count);
    
    if message_count > 0 {
        println!("Average message rate: {:.2} messages/minute", 
                message_count as f64 / (60.0));  // Assuming 1 hour of data
        println!("Success rate: {:.2}%", 
                (message_count as f64 / (message_count + error_count) as f64) * 100.0);
    }
    
    Ok(())
} 