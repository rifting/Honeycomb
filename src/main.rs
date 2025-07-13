use std::{fs::File, io::{BufReader, Read, Write}, path::PathBuf};

use clap::Parser;
use honeycomb::{BinaryXmlDeserializer, Policy, SeekableReader};
use quick_xml::{events::Event, Reader};

// Typically /data/system/users/0.xml
const USER_PROFILE_PATH: &str = "guh.xml";

/// Android device policy editor
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// Name of policy you want to enable/disable
    #[arg(short, long, required_unless_present = "list_policies")]
    policy_name: Option<String>,

    /// Output file name
    #[arg(short, long, required_unless_present = "overwrite")]
    out: Option<String>,

    /// List available policies and exit
    #[arg(long)]
    list_policies: bool,

    // Whether to overwrite the original file
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

    if args.list_policies {
        let policies = get_policy_list(USER_PROFILE_PATH);
        for i in 0..policies.len() {
            println!("{}", policies[i]);
        }
        return;
    } else {
        // For adding a policy, call get_restriction_node_offset to get the restriction offset
        // For removing a policy, use the cleaned policy list struct
        let policy_name = args.policy_name.unwrap();
        let file = File::open(USER_PROFILE_PATH).unwrap();
        let buf_reader = BufReader::new(file);
        let mut seekable_reader = SeekableReader::new(buf_reader);
        let mut output = Vec::new();
        let mut deserializer = BinaryXmlDeserializer::new(&mut seekable_reader, &mut output, true).unwrap();
        let _ = deserializer.deserialize();
        
        // I named this function terribly. It gets all attributes in the ABX/XML, NOT all policies. So we have to clean it
        let uncleaned_policy_list = deserializer.get_policies().to_vec();
        let policy_names = get_policy_list(USER_PROFILE_PATH);
        
        let cleaned_policy_list: Vec<Policy> = uncleaned_policy_list
            .into_iter()
            .filter(|policy| policy_names.contains(&policy.name))
            .collect();

        let mut should_create_policy = true;

        for policy in &cleaned_policy_list {

            // If this resolves to true, then we need to DELETE this policy.

            if policy.name == policy_name {
                should_create_policy = false;
                println!(
                    "Found {} with start offset {} and end offset {}",
                    policy.name, policy.start_offset, policy.end_offset
                );

                let mut buffer = Vec::new();
                let mut file2 = File::open(USER_PROFILE_PATH).unwrap();
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

                println!("Wrote new user profile to {}!", args.out.clone().unwrap());
            }
        }

        if should_create_policy {
            println!("TODO: Policy Creation");
        }
    }
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