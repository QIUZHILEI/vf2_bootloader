use byteorder::{ByteOrder, LittleEndian};
use log::debug;
use lego_device::BlockDevice;

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
        name: &str,
        blk_dev: &mut dyn BlockDevice,
    ) -> Option<(usize, usize)> {
        let mut res = (0, 0);
        let target_name = serialize_name(name);
        let mut lba = self.bpb.root_sector();
        let mut search_num = 0;
        while search_num < self.bpb.sectors_per_cluster {
            let mut buf = [0u8; 512];
            blk_dev.read_block(lba + self.start_lba, &mut buf).unwrap();
            for index in 0..(512 / 32) {
                let start = index * 32;
                if let Some(entry) = DirEntry::deserialize(&buf[start..(start + 32)]) {
                    if entry.is_file() && target_name == entry.name {
                        let cluster = entry.cluster();
                        let sector = self.bpb.cluster_to_sector(cluster);
                        debug!(
                            "kernel is found in disk lba: {}, fat cluster: {}, fat sector: {}, size :{}, name: {}",
                            self.start_lba + sector,
                            cluster,
                            sector,
                            entry.size,
                            name,
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

fn serialize_name(name: &str) -> [u8; 11] {
    let point = name.find(".").unwrap();
    let mut name_ascii = [32u8; 11];
    let name = name.as_bytes();
    name_ascii[..point].copy_from_slice(&name[..point]);
    name_ascii[8..(name.len() + 8 - (point + 1))].copy_from_slice(&name[(point + 1)..]);
    name_ascii
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
