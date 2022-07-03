use gcd_rs::composer::Composer;
use gcd_rs::parser::Parser;
use gcd_rs::record::descriptor::descriptor_data::DescriptorDecoded;
use gcd_rs::record::firmware::FirmwareRecord;
use gcd_rs::{GcdDefaultEndian, Record, Version};
use std::collections::HashMap;
use std::env::args;
use std::fs::File;
use std::io::{BufReader, BufWriter, Read};

struct FwModify {
    //data to inject in this FW
    injects: Vec<Injection>,
    //addr counter used to track the FW current pos
    addr: usize,
    //counter used to verify how much of the firmware need to be received
    size_left: Option<usize>,
    //the FW new checksum value
    sum: u8,
}

struct Injection {
    payload: Vec<u8>,
    pos: usize,
    inject_addr: usize,
    filename: String,
}

fn main() {
    let mut injections = HashMap::new();

    let mut args = args();
    let _ = args.next().expect("Arg 0 missing");
    let fw_file = args.next().expect("Arg 1 fw_file missing");
    let out_file = args.next().expect("Arg 2 out_file missing");

    loop {
        let payload_file = if let Some(x) = args.next() { x } else { break };
        let inject_addr = args.next().expect("Arg 3 inject_addr missing");
        let inject_addr = inject_addr.trim_start_matches("0x");
        let inject_addr = usize::from_str_radix(inject_addr, 16)
            .expect("Invalid inject addr, use hex (0x1234)");
        let inject_fw_id = args.next().expect("Arg 4 inject_fw_id missing");
        let inject_fw_id = inject_fw_id.trim_start_matches("0x");
        let inject_fw_id = u16::from_str_radix(inject_fw_id, 16)
            .expect("Invalid inject fw id, use hex (0x1234)");

        let mut payload = vec![];
        File::open(&payload_file)
            .expect("Unable to open payload file")
            .read_to_end(&mut payload)
            .expect("Unable to read payload file");

        let injection = injections.entry(inject_fw_id).or_insert(FwModify {
            injects: Vec::with_capacity(1),
            addr: 0,
            size_left: None,
            sum: 0,
        });

        injection.injects.push(Injection {
            payload,
            pos: 0,
            inject_addr,
            filename: payload_file.to_string(),
        });
    }

    let fw_file_in = BufReader::new(
        File::open(fw_file).expect("Unable to open source firmware file"),
    );
    let mut parser: Parser<BufReader<File>, GcdDefaultEndian> =
        Parser::new(fw_file_in).expect("Unable to parse source firmware file");

    let fw_file_out = BufWriter::new(
        File::create(out_file).expect("Unable to create final firmware file"),
    );
    let mut composer: Composer<BufWriter<File>, GcdDefaultEndian> =
        Composer::new(fw_file_out)
            .expect("Unable to compose final firmware file");

    let mut last_firmware_size = None;

    loop {
        let record = parser.read_record().unwrap();
        match record {
            Record::Descriptor(mut desc) => {
                //increase the firmware version to force update
                for desc in desc.iter_mut() {
                    match desc.decode() {
                        Some(DescriptorDecoded::FirmwareLen(len)) => {
                            //TODO is not a requirement that Descriptor preced
                            //the firmware data, so we need to check if the
                            //firmware id is correct
                            last_firmware_size = Some(len as usize);
                        }
                        Some(DescriptorDecoded::VersionSw(
                            Version::Simple {
                                ref mut major,
                                ref mut minor,
                            },
                        )) => {
                            *major = major.saturating_add(1);
                            *minor = minor.saturating_add(1);
                        }
                        _ => {}
                    }
                }
                composer.write_record(&Record::Descriptor(desc)).unwrap();
            }
            Record::FirmwareData(FirmwareRecord::Chunk { id, mut data }) => {
                //check if one of the injections will be in this chunk,
                //otherwise just pass the chunk and continue
                let fw_modify = if let Some(fw_modify) = injections.get_mut(&id)
                {
                    fw_modify
                } else {
                    composer
                        .write_record(&Record::FirmwareData(
                            FirmwareRecord::Chunk { id, data },
                        ))
                        .unwrap();
                    continue;
                };
                //update the size left of this fw, if the first block
                match (fw_modify.size_left.as_mut(), last_firmware_size) {
                    //not the first block, just decrease the size that we
                    //received ftom the size left
                    (Some(size_left), _) => {
                        *size_left = size_left.checked_sub(data.len()).unwrap();
                    }
                    //first block, set the size if we found on the last
                    //descriptor record
                    (None, Some(fw_size)) => {
                        fw_modify.size_left = Some(fw_size)
                    }
                    _ => panic!("Unable to find the firmware size"),
                }
                for injection in fw_modify.injects.iter_mut() {
                    let addr_start = fw_modify.addr;
                    let addr_end = fw_modify.addr + data.len();

                    let injection_left =
                        injection.payload.len() - injection.pos;
                    if injection_left > 0 && injection.inject_addr < addr_end {
                        //inject the payload
                        let inject_start =
                            if addr_start >= injection.inject_addr {
                                0
                            } else {
                                injection.inject_addr - addr_start
                            };
                        let inject_len =
                            if inject_start + injection_left > data.len() {
                                data.len() - inject_start
                            } else {
                                injection_left
                            };
                        let inject_end = inject_start + inject_len;
                        data[inject_start..inject_end]
                            .iter_mut()
                            .enumerate()
                            .for_each(|(i, x)| {
                                *x = injection.payload[i + injection.pos]
                            });
                        injection.pos += inject_len;
                    }
                }
                fw_modify.addr += data.len();

                //last chunk
                if fw_modify.size_left.expect("unable to define firmware size")
                    == 0
                {
                    //first check if all injections were consumed
                    for injection in fw_modify.injects.iter_mut() {
                        if injection.payload.len() - injection.pos > 0 {
                            panic!(
                                "Unable to inject the payload {}",
                                injection.filename
                            );
                        }
                    }
                    //replace the checksum
                    let last_byte = data.len() - 1;
                    fw_modify.sum = data[..last_byte]
                        .iter()
                        .fold(fw_modify.sum, |acc, &b| acc.wrapping_add(b));
                    data[last_byte] = fw_modify.sum.wrapping_neg();
                } else {
                    fw_modify.sum = data
                        .iter()
                        .fold(fw_modify.sum, |acc, &b| acc.wrapping_add(b));
                }
                composer
                    .write_record(&Record::FirmwareData(
                        FirmwareRecord::Chunk { id, data },
                    ))
                    .unwrap();
            }
            Record::End => break,
            _ => composer.write_record(&record).unwrap(),
        }
    }
    composer.write_record(&Record::End).unwrap();
}
