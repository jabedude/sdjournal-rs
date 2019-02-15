use std::fs::File;
use byteorder::{LittleEndian, ReadBytesExt};
use std::io::{Read, Result};
use std::env;

mod journal;
use crate::journal::*;

fn load_header(mut file: File) -> Result<JournalHeader> {
    let mut signature = [0u8; 8];
    file.read_exact(&mut signature)?;
    let compatible_flags = file.read_u32::<LittleEndian>()?;
    let incompatible_flags = file.read_u32::<LittleEndian>()?;
    let state = file.read_u8()?;
    let mut reserved = [0u8; 7];
    file.read_exact(&mut reserved)?;
    let mut file_id = [0u8; 16];
    file.read_exact(&mut file_id)?;
    let file_id = sd_id128 { bytes: file_id };
    let mut machine_id = [0u8; 16];
    file.read_exact(&mut machine_id)?;
    let machine_id = sd_id128 { bytes: machine_id };
    let boot_id = file.read_u128::<LittleEndian>()?;
    let seqnum_id = file.read_u128::<LittleEndian>()?;
    let header_size = file.read_u64::<LittleEndian>()?;
    let arena_size = file.read_u64::<LittleEndian>()?;
    let data_hash_table_offset = file.read_u64::<LittleEndian>()?;
    let data_hash_table_size = file.read_u64::<LittleEndian>()?;
    let field_hash_table_offset = file.read_u64::<LittleEndian>()?;
    let field_hash_table_size = file.read_u64::<LittleEndian>()?;
    let tail_object_offset = file.read_u64::<LittleEndian>()?;
    let n_objects = file.read_u64::<LittleEndian>()?;
    let n_entries = file.read_u64::<LittleEndian>()?;
    let tail_entry_seqnum = file.read_u64::<LittleEndian>()?;
    let head_entry_seqnum = file.read_u64::<LittleEndian>()?;
    let entry_array_offset = file.read_u64::<LittleEndian>()?;
    let head_entry_realtime = file.read_u64::<LittleEndian>()?;
    let tail_entry_realtime = file.read_u64::<LittleEndian>()?;
    let tail_entry_monotonic = file.read_u64::<LittleEndian>()?;
    let n_data = file.read_u64::<LittleEndian>()?;
    let n_fields = file.read_u64::<LittleEndian>()?;
    let n_tags = file.read_u64::<LittleEndian>()?;
    let n_entry_arrays = file.read_u64::<LittleEndian>()?;

    Ok(JournalHeader {
        signature: signature,
        compatible_flags: compatible_flags,
        incompatible_flags: incompatible_flags,
        state: state,
        reserved: reserved,
        file_id: file_id,
        machine_id: machine_id,
        boot_id: boot_id,
        seqnum_id: seqnum_id,
        header_size: header_size,
        arena_size: arena_size,
        data_hash_table_offset: data_hash_table_offset,
        data_hash_table_size: data_hash_table_size,
        field_hash_table_offset: field_hash_table_offset,
        field_hash_table_size : field_hash_table_size,
        tail_object_offset: tail_object_offset,
        n_objects: n_objects,
        n_entries: n_entries,
        tail_entry_seqnum: tail_entry_seqnum,
        head_entry_seqnum: head_entry_seqnum,
        entry_array_offset: entry_array_offset,
        head_entry_realtime: head_entry_realtime,
        tail_entry_realtime: tail_entry_realtime,
        tail_entry_monotonic: tail_entry_monotonic,
        /* Added in 187 */
        n_data: n_data,
        n_fields: n_fields,
        /* Added in 189 */
        n_tags: n_tags,
        n_entry_arrays: n_entry_arrays,
    })
}

fn main() {
    let arg: Vec<String> = env::args().collect();
    let mut journal = File::open(&arg[1]).unwrap();
    let h = load_header(journal).unwrap();
    println!("sig: {}", std::str::from_utf8(&h.signature).unwrap());
}
