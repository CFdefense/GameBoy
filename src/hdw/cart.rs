use lazy_static::lazy_static;
use std::collections::HashMap;
use std::fs::File;
use std::io::{self, Read, Seek, SeekFrom};
use std::path::Path;

/*

--TODO--

Bank valid Cart Data From Resource

Load Rom

Error Handle

Print Info

Checksum

*/

struct cartridge_header {
    entry_point: [u8; 4],
    nintendo_logo: [u8; 0x30],
    rom_title: [u8; 16],
    new_lic_code: u16,
    sgb_flag: u8,
    cart_type: u8,
    rom_size: u8,
    ram_size: u8,
    dest_code: u8,
    old_lic_code: u8,
    version: u8,
    checksum: u8,
    global_checksum: u16,
}

struct cartridge {
    file_name: String,
    rom_size: usize,
    rom_data: Vec<u8>,
    rom_header: cartridge_header,
}

impl cartridge {
    pub fn new() -> self {
        Cartidge {
            file_name: String::new(),
            rom_size: 0,
            rom_data: 0,
            rom_header: cartridge_header::new(),
        }
    }
    // Function to load in cartridge
    pub fn load_cart(&mut self, file_path: &str) -> Result<(), String> {
        // Update File Name
        self.file_name = file_path.to_string();

        // Open the cartridge file
        let mut file = File::open(file_path)
            .map_err(|e| format!("Failed to open: {}. Error: {}", file_path, e))?;
        println!("Opened: {}", self.filename);

        // Seek to end of the file to update file size
        file.seek(SeekFrom::End(0))
            .map_err(|e| format!("Error Seeking File: {}", e))?;
        self.rom_size = file
            .metadata()
            .map_err(|e| format!("Error Getting File Length {}", e))?
            .len() as usize;

        // Rewind to start
        file.seek(SeekFrom::Start(0))
            .map_err(|e| format!("Error Rewinding File {}", e))?;

        // Allocate Mem Size
        self.rom_data.resize(self.rom_size, 0);
        file.read_exact(&mut self.rom_data)
            .map_err(|e| format!("Failed to Read Rom Data {}", e))?;

        println!("Cartidge Loaded");

        // Load Header Information
        self.rom_header = cartridge_header {
            new_lic_code: u16::from_le_bytes([self.rom_data[0x0143], self.rom_data[0x0144]]),
            sgb_flag: self.rom_data[0x0146],
            cart_type: self.rom_data[0x0147],
            rom_size: self.rom_data[0x0148],
            ram_size: self.rom_data[0x0149],
            dest_code: self.rom_data[0x014A],
            old_lic_code: self.rom_data[0x014B],
            version: self.rom_data[0x014C],
            checksum: self.rom_data[0x014D],
            global_checksum: u16::from_le_bytes([self.rom_data[0x014E], self.rom_data[0x014F]]),
        };

        // Calculate the actual ROM size per pandocs
        self.rom_size = 32 * 1024 * (1 << self.rom_header.rom_size);

        // Perform Checksum Test
        self.checksum_test()?;

        // Print Cartridge Information
        self.print_info();

        Ok(())
    }

    fn print_info(&self) {
        println!("Cartridge Information:");
        println!(
            "  Title            : {:?}",
            std::str::from_utf8(&self.rom_header.rom_title).unwrap_or("Invalid UTF-8")
        );
        println!("  New License Code : {:#04X}", self.rom_header.new_lic_code);
        println!(
            "  License Name     : {}",
            self.rom_header.license_lookup().unwrap_or("UNKNOWN")
        );
        println!("  SGB Flag         : {:#02X}", self.rom_header.sgb_flag);
        println!(
            "  Cartridge Type   : {:#02X} ({})",
            self.rom_header.cart_type,
            self.rom_header.license_lookup().unwrap_or("UNKNOWN")
        );
        println!("  ROM Size         : {} KB", 32 << self.rom_header.rom_size);
        println!("  RAM Size         : {:#02X}", self.rom_header.ram_size);
        println!("  Destination Code : {:#02X}", self.rom_header.dest_code);
        println!("  Old License Code : {:#02X}", self.rom_header.old_lic_code);
        println!("  Version Number   : {:#02X}", self.rom_header.version);
    }

    fn checksum_test(&self) -> Result<(), String> {
        // Calculate the checksum of the ROM using the specified method
        let mut checksum: u8 = 0;

        // Calculate the checksum from the specified range
        for address in 0x0134..=0x014C {
            checksum = checksum.wrapping_sub(self.rom_data[address] + 1);
        }

        // Check if the calculated checksum matches the stored checksum
        if checksum == self.rom_header.checksum {
            println!("\tChecksum: {:#02X} (PASSED)", checksum);
            Ok(())
        } else {
            Err(format!(
                "\tChecksum: {:#02X} (FAILED, expected: {:#02X})",
                checksum, self.rom_header.checksum
            ))
        }
    }
}

impl cartridge_header {
    // Constructor
    pub fn new() -> Self {
        cartridge_header {
            entry_point: 0,
            nintendo_logo: 0,
            rom_title: 0,
            new_lic_code: 0,
            sgb_flag: 0,
            cart_type: 0,
            rom_size: 0,
            ram_size: 0,
            dest_code: 0,
            old_lic_code: 0,
            version: 0,
            checksum: 0,
            global_checksum: 0,
        }
    }
    // Function to lookup publisher code
    fn license_lookup(self) -> Option<&'static str> {
        match PUBLISHER_CODES.get(&format!("{:02X}", self.old_lic_code)) {
            Some(&publisher) => publisher, // Return the found publisher
            None => "UNKNOWN",             // Return "UNKNOWN" if not found
        }
    }

    // Function to lookup cartridge type
    fn license_lookup(self) -> Option<&'static str> {
        match PUBLISHER_CODES.get(&format!("{:02X}", self.cart_type)) {
            Some(&cart_type) => cart_type, // Return the found publisher
            None => "UNKNOWN",             // Return "UNKNOWN" if not found
        }
    }
}

lazy_static! {
    static ref PUBLISHER_CODES: HashMap<&'static str, &'static str> = {
        let mut map = HashMap::new();
        map.insert("00", "None");
        map.insert("01", "Nintendo Research & Development 1");
        map.insert("08", "Capcom");
        map.insert("13", "EA (Electronic Arts)");
        map.insert("18", "Hudson Soft");
        map.insert("19", "B-AI");
        map.insert("20", "KSS");
        map.insert("22", "Planning Office WADA");
        map.insert("24", "PCM Complete");
        map.insert("25", "San-X");
        map.insert("28", "Kemco");
        map.insert("29", "SETA Corporation");
        map.insert("30", "Viacom");
        map.insert("31", "Nintendo");
        map.insert("32", "Bandai");
        map.insert("33", "Ocean Software/Acclaim Entertainment");
        map.insert("34", "Konami");
        map.insert("35", "HectorSoft");
        map.insert("37", "Taito");
        map.insert("38", "Hudson Soft");
        map.insert("39", "Banpresto");
        map.insert("41", "Ubi Soft");
        map.insert("42", "Atlus");
        map.insert("44", "Malibu Interactive");
        map.insert("46", "Angel");
        map.insert("47", "Bullet-Proof Software");
        map.insert("49", "Irem");
        map.insert("50", "Absolute");
        map.insert("51", "Acclaim Entertainment");
        map.insert("52", "Activision");
        map.insert("53", "Sammy USA Corporation");
        map.insert("54", "Konami");
        map.insert("55", "Hi Tech Expressions");
        map.insert("56", "LJN");
        map.insert("57", "Matchbox");
        map.insert("58", "Mattel");
        map.insert("59", "Milton Bradley Company");
        map.insert("60", "Titus Interactive");
        map.insert("61", "Virgin Games Ltd.");
        map.insert("64", "Lucasfilm Games");
        map.insert("67", "Ocean Software");
        map.insert("69", "EA (Electronic Arts)");
        map.insert("70", "Infogrames");
        map.insert("71", "Interplay Entertainment");
        map.insert("72", "Broderbund");
        map.insert("73", "Sculptured Software");
        map.insert("75", "The Sales Curve Limited");
        map.insert("78", "THQ");
        map.insert("79", "Accolade");
        map.insert("80", "Misawa Entertainment");
        map.insert("83", "lozc");
        map.insert("86", "Tokuma Shoten");
        map.insert("87", "Tsukuda Original");
        map.insert("91", "Chunsoft Co.");
        map.insert("92", "Video System");
        map.insert("93", "Ocean Software/Acclaim Entertainment");
        map.insert("95", "Varie");
        map.insert("96", "Yonezawa/s’pal");
        map.insert("97", "Kaneko");
        map.insert("99", "Pack-In-Video");
        map.insert("9H", "Bottom Up");
        map.insert("A4", "Konami (Yu-Gi-Oh!)");
        map.insert("BL", "MTO");
        map.insert("DK", "Kodansha");
        map
    };
}

lazy_static! {
    static ref ROM_TYPES: HashMap<u8, &'static str> = {
        let mut map = HashMap::new();
        map.insert(0x00, "ROM ONLY");
        map.insert(0x01, "MBC1");
        map.insert(0x02, "MBC1+RAM");
        map.insert(0x03, "MBC1+RAM+BATTERY");
        map.insert(0x05, "MBC2");
        map.insert(0x06, "MBC2+BATTERY");
        map.insert(0x08, "ROM+RAM");
        map.insert(0x09, "ROM+RAM+BATTERY");
        map.insert(0x0B, "MMM01");
        map.insert(0x0C, "MMM01+RAM");
        map.insert(0x0D, "MMM01+RAM+BATTERY");
        map.insert(0x0F, "MBC3+TIMER+BATTERY");
        map.insert(0x10, "MBC3+TIMER+RAM+BATTERY");
        map.insert(0x11, "MBC3");
        map.insert(0x12, "MBC3+RAM");
        map.insert(0x13, "MBC3+RAM+BATTERY");
        map.insert(0x19, "MBC5");
        map.insert(0x1A, "MBC5+RAM");
        map.insert(0x1B, "MBC5+RAM+BATTERY");
        map.insert(0x1C, "MBC5+RUMBLE");
        map.insert(0x1D, "MBC5+RUMBLE+RAM");
        map.insert(0x1E, "MBC5+RUMBLE+RAM+BATTERY");
        map.insert(0x20, "MBC6");
        map.insert(0x22, "MBC7+SENSOR+RUMBLE+RAM+BATTERY");
        map.insert(0xFC, "POCKET CAMERA");
        map.insert(0xFD, "BANDAI TAMA5");
        map.insert(0xFE, "HuC3");
        map.insert(0xFF, "HuC1+RAM+BATTERY");
        map
    };
}
