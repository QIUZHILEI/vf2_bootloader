use core::fmt::Display;

use byteorder::{ByteOrder, LittleEndian};
use lego_device::BlockDevice;
use log::{debug, error};

#[derive(Default, Debug, Clone)]
#[allow(unused)]
struct BpbSector {
    bytes_per_sector: u16,
    sectors_per_cluster: u8,
    reserved_sectors: u16,
    fats: u8,
    root_entries: u16,
    total_sectors_32: u32,
    sectors_per_fat_32: u32,
    root_dir_first_cluster: u32,
    fs_info_sector: u16,
    backup_boot_sector: u16,
    volume_id: u32,
    volume_label: [u8; 11],
    fs_type_label: [u8; 8],
}

impl BpbSector {
    pub(crate) fn deserialize(sector: &[u8]) -> Self {
        assert!(sector.len() >= 512);
        let bytes_per_sector = LittleEndian::read_u16(&sector[11..13]);
        let sectors_per_cluster = sector[13];
        let reserved_sectors = LittleEndian::read_u16(&sector[14..16]);
        let fats = sector[16];
        let root_entries = LittleEndian::read_u16(&sector[17..19]);
        let total_sectors_32 = LittleEndian::read_u32(&sector[32..36]);
        let sectors_per_fat_32 = LittleEndian::read_u32(&sector[36..40]);
        let root_dir_first_cluster = LittleEndian::read_u32(&sector[44..48]);
        let fs_info_sector = LittleEndian::read_u16(&sector[48..50]);
        let backup_boot_sector = LittleEndian::read_u16(&sector[50..52]);
        let volume_id = LittleEndian::read_u32(&sector[67..71]);
        let mut volume_label = [0u8; 11];
        volume_label.copy_from_slice(&sector[71..82]);
        let mut fs_type_label = [0u8; 8];
        fs_type_label.copy_from_slice(&sector[82..90]);
        assert_eq!((sector[510], sector[511]), (0x55, 0xaa));

        Self {
            bytes_per_sector,
            sectors_per_cluster,
            reserved_sectors,
            fats,
            root_entries,
            total_sectors_32,
            sectors_per_fat_32,
            root_dir_first_cluster,
            fs_info_sector,
            backup_boot_sector,
            volume_id,
            volume_label,
            fs_type_label,
        }
    }
}

impl BpbSector {
    fn root_sector(&self) -> usize {
        (self.reserved_sectors as u32 + self.fats as u32 * self.sectors_per_fat_32) as usize
    }

    fn cluster_to_sector(&self, cluster: usize) -> usize {
        self.root_sector()
            + (cluster - self.root_dir_first_cluster as usize) * (self.sectors_per_cluster as usize)
    }
}
#[derive(Debug)]
pub(crate) struct Volume {
    start_lba: usize,
    bpb: BpbSector,
}

impl Volume {
    pub(crate) fn new(start_lba: usize) -> Self {
        Self {
            start_lba,
            bpb: BpbSector::default(),
        }
    }

    pub(crate) fn init_bpb(&mut self, sector: &[u8]) {
        self.bpb = BpbSector::deserialize(sector);
    }

    pub(crate) fn find(
        &self,
        name: &[u8],
        blk_dev: &mut dyn BlockDevice,
    ) -> Option<(usize, usize)> {
        let mut res = (0, 0);
        let target = FileName::from_slice(name);
        if target.is_none() {
            error!("The file name entered is invalid!");
            return None;
        }
        let target = target.unwrap();
        let mut lba = self.bpb.root_sector();
        let mut search_num = 0;
        while search_num < self.bpb.sectors_per_cluster {
            let mut buf = [0u8; 512];
            blk_dev.read_block(lba + self.start_lba, &mut buf).unwrap();
            for index in 0..(512 / 32) {
                let start = index * 32;
                if let Some(entry) = DirEntry::deserialize(&buf[start..(start + 32)]) {
                    if entry.is_file() && target.0 == entry.name {
                        let cluster = entry.cluster();
                        let sector = self.bpb.cluster_to_sector(cluster);
                        debug!(
                            "kernel is found in disk lba: {}, fat cluster: {}, fat sector: {}, size :{}, name: {}",
                            self.start_lba + sector,
                            cluster,
                            sector,
                            entry.size,
                            target,
                        );
                        res.0 = self.start_lba + sector;
                        res.1 = entry.size as usize;
                        break;
                    }
                }
            }
            lba += 1;
            search_num += 1;
        }
        if res == (0, 0) {
            None
        } else {
            Some(res)
        }
    }
}

const CAPITAL: u8 = 65;
const SMALL: u8 = 97;
const POINT: u8 = 46;
const DIGIT: u8 = 48;

fn valid_char(byte: u8) -> Option<u8> {
    if (byte >= DIGIT && byte <= DIGIT + 9) || (byte >= CAPITAL && byte <= CAPITAL + 25) {
        Some(byte)
    } else if byte >= SMALL && byte < SMALL + 26 {
        Some(byte - (SMALL - CAPITAL))
    } else {
        None
    }
}

pub const FILE_NAME_LEN: usize = 11;
struct FileName([u8; FILE_NAME_LEN]);

impl FileName {
    fn from_slice(slice: &[u8]) -> Option<Self> {
        let mut name = [32u8; FILE_NAME_LEN];
        if slice[0] == POINT || slice.len() > FILE_NAME_LEN + 1 {
            return None;
        }
        let mut point_index = 8;
        for index in 0..8 {
            if index == slice.len() {
                return Some(Self(name));
            }
            let byte = slice[index];
            if byte == POINT {
                point_index = index;
                break;
            }
            match valid_char(byte) {
                Some(b) => name[index] = b,
                None => return None,
            }
        }
        if slice.len() - 1 - point_index > 3 {
            return None;
        }
        let mut ext_index = 8;
        for index in (point_index + 1)..slice.len() {
            let byte = slice[index];
            match valid_char(byte) {
                Some(b) => name[ext_index] = b,
                None => return None,
            }
            ext_index += 1;
        }
        Some(Self(name))
    }
}

impl Display for FileName {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        for index in 0..8 {
            let byte = self.0[index];
            if byte == 0 {
                break;
            }
            write!(f, "{}", byte as char).unwrap();
        }
        for index in 8..self.0.len() {
            let byte = self.0[index];
            if byte == 0 {
                break;
            }
            write!(f, "{}", byte as char).unwrap();
        }
        Ok(())
    }
}
#[derive(Debug)]
struct DirEntry {
    name: [u8; 11],
    cluster_h: u16,
    cluster_l: u16,
    size: u32,
}

impl DirEntry {
    fn deserialize(bytes: &[u8]) -> Option<Self> {
        assert!(bytes.len() == 32);
        let bytes = {
            let mut non_bytes = [0u8; 32];
            non_bytes.copy_from_slice(bytes);
            non_bytes
        };
        if bytes == [0u8; 32] {
            return None;
        }
        let mut name = [0u8; 11];
        name.copy_from_slice(&bytes[0..11]);
        Some(Self {
            name,
            cluster_h: LittleEndian::read_u16(&bytes[20..22]),
            cluster_l: LittleEndian::read_u16(&bytes[26..28]),
            size: LittleEndian::read_u32(&bytes[28..]),
        })
    }

    fn is_file(&self) -> bool {
        self.size != 0
    }

    fn cluster(&self) -> usize {
        self.cluster_l as usize | (self.cluster_h as usize) << u16::BITS
    }
}
