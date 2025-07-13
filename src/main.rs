use std::{fs::File, io::{BufReader, Read, Write}, path::PathBuf, string};

use clap::Parser;
use honeycomb::{BinaryXmlDeserializer, Policy, SeekableReader};
use quick_xml::{events::Event, Reader};

/// Android device policy editor
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// Name of policy you want to enable/disable
    #[arg(short, long, required_unless_present = "list_policies")]
    policy_name: Option<String>,

    /// Input file. For the primary user on android devices, this is typically /data/system/users/0.xml
    #[arg(long, default_value = "/data/system/users/0.xml")]
    profile_path: String,

    /// Output file name
    #[arg(short, long, required_unless_present_any(&["overwrite", "list_policies"]))]
    out: Option<String>,

    /// List available policies and exit
    #[arg(long)]
    list_policies: bool,

    /// Pass this argument to overwrite the original file
    #[arg(long)]
    overwrite: bool,
}

/*
    Roadmap

    Remove policy func
    Add policy func

*/
fn main() {
    let args = Args::parse();
    let user_profile_path = args.profile_path;
    if args.list_policies {
        let policies = get_policy_list(&user_profile_path);
        for i in 0..policies.len() {
            println!("{}", policies[i]);
        }
        return;
    } else {
        // For adding a policy, call get_restriction_node_offset to get the restriction offset
        // For removing a policy, use the cleaned policy list struct
        let policy_name = args.policy_name.unwrap();
        let file = File::open(&user_profile_path).unwrap();
        let buf_reader = BufReader::new(file);
        let mut seekable_reader = SeekableReader::new(buf_reader);
        let mut output = Vec::new();
        let mut deserializer = BinaryXmlDeserializer::new(&mut seekable_reader, &mut output, true).unwrap();
        let _ = deserializer.deserialize();
        
        // I named this function terribly. It gets all attributes in the ABX/XML, NOT all policies. So we have to clean it
        let uncleaned_policy_list = deserializer.get_policies().to_vec();
        let policy_names = get_policy_list(&user_profile_path);
        
        let cleaned_policy_list: Vec<Policy> = uncleaned_policy_list
            .into_iter()
            .filter(|policy| policy_names.contains(&policy.name))
            .collect();

        let mut should_create_policy = true;

        for policy in &cleaned_policy_list {

            // If this resolves to true, then we need to DELETE this policy.

            if policy.name == policy_name {
                should_create_policy = false;
                println!("REMOVING the {} policy", policy.name);
                println!();
                println!(
                    "Found {} with start offset {} and end offset {}",
                    policy.name, policy.start_offset, policy.end_offset
                );

                let mut buffer = Vec::new();
                let mut file2 = File::open(&user_profile_path).unwrap();
                file2.read_to_end(&mut buffer).unwrap();

                buffer.drain(policy.start_offset as usize..policy.end_offset as usize);

                // Decrement the fifth last byte by one. I have no idea what this represents!
                // But when we remove a policy, this must go down too.
                let len = buffer.len();
                if len >= 5 {
                    buffer[len - 5] = buffer[len - 5].wrapping_sub(1);
                }

                let mut new_file = File::create(args.out.clone().unwrap()).unwrap();
                let _ = new_file.write_all(&buffer);

                println!("Successfully disabled the {} policy", policy.name);
                println!("Wrote XML without policy to {}!", args.out.clone().unwrap());
            }
        }

        if should_create_policy {
            println!("CREATING the {} policy", policy_name);
            let offset = deserializer.get_restriction_node_offset();
            let policy_bytes = policy_to_bytes(&policy_name);
            let mut buffer = Vec::new();
            let mut file2 = File::open(user_profile_path).unwrap();
            file2.read_to_end(&mut buffer).unwrap();

            buffer.splice(
                *offset as usize..*offset as usize,
                policy_bytes,
            );

            // Increment the fifth last byte by one.
            let len = buffer.len();
            if len >= 5 {
                buffer[len - 5] = buffer[len - 5].wrapping_add(1);
            }

            let mut new_file = File::create(args.out.clone().unwrap()).unwrap();
            let _ = new_file.write_all(&buffer);

            println!("Successfully added the {} policy", policy_name);
            println!();
            println!("Wrote XML with the new policy to {}!", args.out.clone().unwrap());
        }
        println!();
        println!("You may want to double check that this XML matches your expectations.");
        println!("Watch out for any syntax errors that the ABX -> XML conversion caused.");
        println!("{}", get_readable_xml(args.out.clone().unwrap()));
    }
}

fn policy_to_bytes(policy_name: &str) -> Vec<u8> {
    let mut serialized_policy_node = Vec::new();
    const POLICY_NODE_BYTES: [u8; 3] = [0xCF, 0xFF, 0xFF];

    serialized_policy_node.extend_from_slice(&POLICY_NODE_BYTES);
    let name_len = policy_name.len() as u16;
    serialized_policy_node.extend_from_slice(&name_len.to_be_bytes());
    serialized_policy_node.extend_from_slice(policy_name.as_bytes());

    // for byte in &serialized_policy_node {
    //     print!("{:02X} ", byte);
    // }
    return serialized_policy_node;
}

fn get_readable_xml(path: String) -> String {
    let file = File::open(path).unwrap();
    let buf_reader = BufReader::new(file);
    let mut seekable_reader = SeekableReader::new(buf_reader);
    let mut output = Vec::new();
    let mut deserializer = BinaryXmlDeserializer::new(&mut seekable_reader, &mut output, false).unwrap();
    let _ = deserializer.deserialize();

    // human readable form of the ABX file
    let xml_str = String::from_utf8(output).unwrap();

    return xml_str;
}

fn get_policy_list(abx_path: &str) -> Vec<String> {
    let mut list_output: Vec<String> = Vec::new();
    let file = File::open(abx_path).unwrap();
    let buf_reader = BufReader::new(file);
    let mut seekable_reader = SeekableReader::new(buf_reader);
    let mut output = Vec::new();
    let mut deserializer = BinaryXmlDeserializer::new(&mut seekable_reader, &mut output, false).unwrap();
    let _ = deserializer.deserialize();

    // human readable form of the ABX file
    let xml_str = String::from_utf8(output).unwrap();

    let mut reader = Reader::from_str(&xml_str);

    let mut buf = Vec::new();

    let mut is_correct_policy_node = false;

    loop {
        let event = reader.read_event_into(&mut buf).unwrap();
        match event {
            Event::Eof => break,
            Event::Start(e) => {
                // Check if this is the <restrictions> inside device_policy_local_restrictions
                let event_name = e.name();

                match event_name.as_ref() {
                    b"restrictions" => {
                        if is_correct_policy_node {
                            for attr in e.attributes().flatten() {
                                let key = String::from_utf8_lossy(attr.key.as_ref()).to_string();
                                list_output.push(key);
                            }
                        }
                    },
                    b"restrictions_user" => {
                        // We do this to ensure that these are the restrictions inside of <restrictions_user />
                        is_correct_policy_node = true
                    },
                    _ => {}
                }
            }
            _ => {}
        }
        buf.clear();
    }

    return list_output;

}