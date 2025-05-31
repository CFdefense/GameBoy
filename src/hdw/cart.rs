use lazy_static::lazy_static;
use std::collections::HashMap;
use std::fs::File;
use std::io::{Read, Seek, SeekFrom};
/*

--TODO--

Bank valid Cart Data From Resource

Load Rom

Error Handle

Print Info

Checksum

*/

struct CartridgeHeader {
    //entry_point: [u8; 4],
    //nintendo_logo: [u8; 0x30],
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

pub struct Cartridge {
    file_name: String,
    rom_size: usize,
    rom_data: Vec<u8>,
    rom_header: CartridgeHeader,

    // MBC Type 1 & 3
    ram_enabled: bool,
    ram_banking: bool,
    rom_bank_x: usize, // Index into ROM data for current bank
    banking_mode: u8, 
    rom_bank_value: u8,
    ram_bank_value: u8,
    ram_bank: usize, // Index into ram_banks
    ram_banks: [Option<Vec<u8>>; 16], // Each bank is 8KB when allocated
    battery: bool,
    need_save: bool,
    
    // MBC3 RTC (Real Time Clock) support
    rtc_registers: [u8; 5], // RTC S, M, H, DL, DH (0x08-0x0C)
    rtc_latched: [u8; 5],   // Latched RTC values
    rtc_latch_state: u8,    // For latch sequence (0x00 -> 0x01)
    rtc_selected: bool,     // True if RTC register selected instead of RAM
    rtc_register_select: u8, // Which RTC register (0x08-0x0C)
    
    // RTC timing (simplified - real implementation would use system time)
    rtc_last_time: std::time::SystemTime,
}

impl Cartridge {
    pub fn new() -> Cartridge {
        let cartridge = Cartridge {
            file_name: String::new(),
            rom_size: 0,
            rom_data: Vec::<u8>::new(),
            rom_header: CartridgeHeader::new(),
            ram_enabled: false,
            ram_banking: false,
            rom_bank_x: 0,
            banking_mode: 0,
            rom_bank_value: 0,
            ram_bank_value: 0,
            ram_bank: 0,
            ram_banks: std::array::from_fn(|_| None),
            battery: false,
            need_save: false,
            rtc_registers: [0; 5],
            rtc_latched: [0; 5],
            rtc_latch_state: 0,
            rtc_selected: false,
            rtc_register_select: 0,
            rtc_last_time: std::time::SystemTime::now(),
        };
        cartridge
    }

    pub fn cart_setup_banking(&mut self) {
        for i in 0..16 {
            self.ram_banks[i] = None;

            if (self.rom_header.ram_size == 2 && i == 0) || 
               (self.rom_header.ram_size == 3 && i < 4) || 
               (self.rom_header.ram_size == 4 && i < 16) || 
               (self.rom_header.ram_size == 5 && i < 8) {
                // Allocate 8KB (0x2000 bytes) for each RAM bank
                self.ram_banks[i] = Some(vec![0; 0x2000]);
            }
        }

        self.ram_bank = 0; // Point to first bank
        self.rom_bank_x = 0x4000; // ROM bank 1 starts at 0x4000
        
        // For MBC1, initialize with proper defaults
        if self.cart_mbc1() {
            self.ram_enabled = false; // RAM starts disabled
            self.ram_banking = true;  // Enable RAM banking by default
        }
        
        // For MBC3, initialize with proper defaults
        if self.cart_mbc3() {
            self.ram_enabled = false; // RAM starts disabled
            self.ram_banking = true;  // Enable RAM banking by default
            self.rtc_selected = false; // Start with RAM selected
            self.rtc_register_select = 0x08; // Default to seconds register
            
            // Initialize RTC if supported
            if self.cart_has_rtc() {
                self.update_rtc_time();
            }
        }
    }

    pub fn cart_load_battery(&mut self) {
        if self.ram_banks[self.ram_bank].is_none() {
            return;
        }

        // Extract filename without path
        let filename = std::path::Path::new(&self.file_name)
            .file_name()
            .unwrap_or_else(|| std::ffi::OsStr::new(&self.file_name))
            .to_string_lossy();
        
        let save_file_path = format!("saves/{}.battery", filename);
        
        if let Ok(save_data) = std::fs::read(&save_file_path) {
            println!("Loading battery save: {}", save_file_path);
            
            if let Some(ref mut ram_bank) = self.ram_banks[self.ram_bank] {
                if save_data.len() >= 0x2000 {
                    ram_bank[..0x2000].copy_from_slice(&save_data[..0x2000]);
                } else {
                    // If save file is smaller, copy what we can
                    let copy_len = save_data.len().min(ram_bank.len());
                    ram_bank[..copy_len].copy_from_slice(&save_data[..copy_len]);
                }
            }
        } else {
            println!("FAILED TO OPEN: {}", save_file_path);
        }
    }

    pub fn cart_save_battery(&mut self) {
        if self.ram_banks[self.ram_bank].is_none() {
            return;
        }

        // Create saves directory if it doesn't exist
        if let Err(e) = std::fs::create_dir_all("saves") {
            println!("Failed to create saves directory: {}", e);
            return;
        }

        // Extract filename without path
        let filename = std::path::Path::new(&self.file_name)
            .file_name()
            .unwrap_or_else(|| std::ffi::OsStr::new(&self.file_name))
            .to_string_lossy();
        
        let save_file_path = format!("saves/{}.battery", filename);
        
        if let Some(ref ram_bank) = self.ram_banks[self.ram_bank] {
            // Save only 8KB (0x2000 bytes) from current RAM bank
            let save_data = &ram_bank[..0x2000];
            
            if let Err(e) = std::fs::write(&save_file_path, save_data) {
                println!("FAILED TO OPEN: {}", save_file_path);
                println!("Error: {}", e);
            } else {
                println!("Battery saved: {}", save_file_path);
                self.need_save = false;
            }
        }
    }

    // Function to load in cartridge
    pub fn load_cart(&mut self, file_path: &str) -> Result<(), String> {
        // Update File Name
        self.file_name = file_path.to_string();

        // Open the cartridge file
        let mut file = File::open(file_path)
            .map_err(|e| format!("Failed to open: {}. Error: {}", file_path, e))?;
        println!("Opened: {}", self.file_name);

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

        self.battery = self.cart_battery();
        self.need_save = false;

        println!("Cartidge Loaded");

        // Load Header Information
        self.rom_header = CartridgeHeader {
            //entry_point: [0; 4],
            //nintendo_logo: [0; 0x30],
            rom_title: self.rom_data[0x0134..0x0144]
                .try_into()
                .expect("Failed to read ROM title"),
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

        // Setup Banking
        self.cart_setup_banking();

        // Perform Checksum Test
        self.checksum_test()?;

        // Print Cartridge Information
        self.print_info();

        // Load Battery
        if self.battery {
            self.cart_load_battery();
        }

        Ok(())
    }

    fn print_info(&self) {
        println!("Cartridge Information:");
        println!(
            "  Title            : {:?}",
            std::str::from_utf8(&self.rom_header.rom_title)
                .unwrap_or("Invalid UTF-8")
                .trim_end_matches('\0')
        );
        println!(
            "  New License Code : {:#04X} ({})",
            self.rom_header.new_lic_code,
            self.rom_header.new_license_lookup().unwrap_or("UNKNOWN")
        );
        println!("  SGB Flag         : {:#02X}", self.rom_header.sgb_flag);
        println!(
            "  Cartridge Type   : {:#02X} ({})",
            self.rom_header.cart_type,
            self.rom_header.cart_type_lookup().unwrap_or("UNKNOWN")
        );
        println!("  ROM Size         : {} KB", 32 << self.rom_header.rom_size);
        println!("  RAM Size         : {:#02X}", self.rom_header.ram_size);
        println!(
            "  Destination Code : {:#02X} ({})",
            self.rom_header.dest_code,
            if self.rom_header.dest_code == 0x00 {
                "Japan and possibly overseas"
            } else {
                "Overseas only"
            }
        );
        println!(
            "  Old Licensee Code: {:#02X} ({})",
            self.rom_header.old_lic_code,
            self.rom_header.old_license_lookup().unwrap_or("UNKNOWN")
        );
        println!("  Version Number   : {:#02X}", self.rom_header.version);
        println!(
            "  Global Checksum  : {:#02X}",
            self.rom_header.global_checksum
        );
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

    // Method to read a byte at an address
    pub fn read_byte(&mut self, address: u16) -> u8 {
        if address < 0x4000 { 
            return self.rom_data[address as usize];
        }

        // For MBC3, handle RTC time updates
        if self.cart_mbc3() && self.cart_has_rtc() {
            self.update_rtc_time();
        }

        // For non-MBC games, just read from ROM directly
        if !self.cart_mbc1() && !self.cart_mbc3() {
            let index = address as usize;
            if index < self.rom_data.len() {
                return self.rom_data[index];
            } else {
                return 0xFF; // Out of bounds
            }
        }

        // MBC1 and MBC3 logic below
        if (address & 0xE000) == 0xA000 {
            if !self.ram_enabled {
                return 0xFF;
            }

            // MBC3: Check if RTC register is selected
            if self.cart_mbc3() && self.rtc_selected {
                let rtc_index = (self.rtc_register_select - 0x08) as usize;
                if rtc_index < 5 {
                    return self.rtc_latched[rtc_index];
                }
                return 0xFF;
            }

            if !self.ram_banking {
                return 0xFF;
            }

            if let Some(ref ram_bank) = self.ram_banks[self.ram_bank] {
                return ram_bank[address as usize - 0xA000];
            }
            return 0xFF;
        }
        
        // ROM bank 1+ access for MBC1 and MBC3
        let rom_address = self.rom_bank_x + (address as usize - 0x4000);
        if rom_address < self.rom_data.len() {
            return self.rom_data[rom_address];
        } else {
            return 0xFF; // Out of bounds
        }
    }

    // Method to write a value to an address
    pub fn write_byte(&mut self, address: u16, mut value: u8) {
        if !self.cart_mbc1() && !self.cart_mbc3() {
            return;
        }

        if address < 0x2000 {
            // RAM and Timer Enable (MBC1 & MBC3)
            self.ram_enabled = (value & 0xF) == 0xA;
        }

        if (address & 0xE000) == 0x2000 {
            // ROM bank number (MBC1 & MBC3)
            if value == 0 {
                value = 1;
            }

            if self.cart_mbc1() {
                value &= 0b11111; // MBC1: 5 bits
            } else if self.cart_mbc3() {
                value &= 0b1111111; // MBC3: 7 bits (supports banks 0x01-0x7F)
            }

            self.rom_bank_value = value;
            self.rom_bank_x = 0x4000 * self.rom_bank_value as usize;
        }

        if (address & 0xE000) == 0x4000 {
            // RAM bank number or RTC register select (MBC1 & MBC3)
            if self.cart_mbc3() {
                if value <= 0x07 {
                    // RAM bank selection
                    self.ram_bank_value = value & 0b1111;
                    self.rtc_selected = false;
                    
                    if self.ram_banking {
                        if self.cart_needs_save() {
                            self.cart_save_battery();
                        }
                        self.ram_bank = self.ram_bank_value as usize;
                    }
                } else if value >= 0x08 && value <= 0x0C {
                    // RTC register selection
                    self.rtc_register_select = value;
                    self.rtc_selected = true;
                }
            } else if self.cart_mbc1() {
                // MBC1 RAM bank handling
                self.ram_bank_value = value & 0b1111;
                
                if self.ram_banking {
                    if self.cart_needs_save() {
                        self.cart_save_battery();
                    }
                    self.ram_bank = self.ram_bank_value as usize;
                }
            }
        }

        if (address & 0xE000) == 0x6000 {
            if self.cart_mbc1() {
                // Banking mode select (MBC1)
                self.banking_mode = value & 1;
                self.ram_banking = self.banking_mode != 0;

                if self.ram_banking {
                    if self.cart_needs_save() {
                        self.cart_save_battery();
                    }
                    self.ram_bank = self.ram_bank_value as usize;
                }
            } else if self.cart_mbc3() {
                // Latch Clock Data (MBC3)
                if self.cart_has_rtc() {
                    if self.rtc_latch_state == 0x00 && value == 0x01 {
                        // Latch current RTC values
                        self.rtc_latched = self.rtc_registers;
                    }
                    self.rtc_latch_state = value;
                }
            }
        }

        if (address & 0xE000) == 0xA000 {
            if !self.ram_enabled {
                return;
            }

            // MBC3: Check if writing to RTC register
            if self.cart_mbc3() && self.rtc_selected {
                let rtc_index = (self.rtc_register_select - 0x08) as usize;
                if rtc_index < 5 {
                    self.rtc_registers[rtc_index] = value;
                    // Also update the latched value
                    self.rtc_latched[rtc_index] = value;
                }
                return;
            }

            // Regular RAM write
            if let Some(ref mut ram_bank) = self.ram_banks[self.ram_bank] {
                let ram_address = (address - 0xA000) as usize;
                if ram_address < ram_bank.len() {
                    ram_bank[ram_address] = value;

                    if self.battery {
                        self.need_save = true;
                    }
                }
            }
        }
    }

    pub fn cart_needs_save(&self) -> bool {
        self.need_save
    }

    pub fn cart_battery(&self) -> bool {
        match self.rom_header.cart_type {
            0x03 | 0x06 | 0x09 | 0x0D | // MBC1+RAM+BATTERY, MBC2+BATTERY, ROM+RAM+BATTERY, MMM01+RAM+BATTERY
            0x0F | 0x10 | 0x13 => true, // MBC3+TIMER+BATTERY, MBC3+TIMER+RAM+BATTERY, MBC3+RAM+BATTERY
            _ => false,
        }
    }

    pub fn cart_mbc1(&self) -> bool {
        self.rom_header.cart_type == 0x01 || self.rom_header.cart_type == 0x02 || self.rom_header.cart_type == 0x03
    }

    pub fn cart_mbc3(&self) -> bool {
        match self.rom_header.cart_type {
            0x0F | 0x10 | 0x11 | 0x12 | 0x13 => true,
            _ => false,
        }
    }
    
    pub fn cart_has_rtc(&self) -> bool {
        match self.rom_header.cart_type {
            0x0F | 0x10 => true, // MBC3+TIMER+BATTERY, MBC3+TIMER+RAM+BATTERY
            _ => false,
        }
    }

    // Update RTC registers based on elapsed time
    fn update_rtc_time(&mut self) {
        if !self.cart_has_rtc() {
            return;
        }
        
        let now = std::time::SystemTime::now();
        if let Ok(elapsed) = now.duration_since(self.rtc_last_time) {
            let elapsed_seconds = elapsed.as_secs();
            if elapsed_seconds > 0 {
                // Add elapsed seconds to RTC
                let mut total_seconds = self.rtc_registers[0] as u64; // Seconds
                total_seconds += elapsed_seconds;
                
                // Handle overflow from seconds to minutes
                if total_seconds >= 60 {
                    let minutes = total_seconds / 60;
                    self.rtc_registers[0] = (total_seconds % 60) as u8;
                    
                    let total_minutes = self.rtc_registers[1] as u64 + minutes;
                    if total_minutes >= 60 {
                        let hours = total_minutes / 60;
                        self.rtc_registers[1] = (total_minutes % 60) as u8;
                        
                        let total_hours = self.rtc_registers[2] as u64 + hours;
                        if total_hours >= 24 {
                            let days = total_hours / 24;
                            self.rtc_registers[2] = (total_hours % 24) as u8;
                            
                            // Handle day counter (9 bits total)
                            let mut day_counter = ((self.rtc_registers[4] & 0x01) as u16) << 8 | self.rtc_registers[3] as u16;
                            day_counter = day_counter.wrapping_add(days as u16);
                            
                            // Check for overflow
                            if day_counter > 0x1FF {
                                self.rtc_registers[4] |= 0x80; // Set carry bit
                                day_counter &= 0x1FF; // Keep only 9 bits
                            }
                            
                            self.rtc_registers[3] = (day_counter & 0xFF) as u8;
                            self.rtc_registers[4] = (self.rtc_registers[4] & 0xFE) | ((day_counter >> 8) & 0x01) as u8;
                        } else {
                            self.rtc_registers[2] = total_hours as u8;
                        }
                    } else {
                        self.rtc_registers[1] = total_minutes as u8;
                    }
                } else {
                    self.rtc_registers[0] = total_seconds as u8;
                }
                
                self.rtc_last_time = now;
            }
        }
    }
}

impl CartridgeHeader {
    // Constructor
    pub fn new() -> CartridgeHeader {
        let cartridge_header = CartridgeHeader {
            //entry_point: [0; 4],
            //nintendo_logo: [0; 0x30],
            rom_title: [0; 16],
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
        };
        cartridge_header
    }
    // Function to lookup publisher code
    fn new_license_lookup(&self) -> Option<&'static str> {
        match NEW_LICENSEE_CODES.get(&format!("{:02X}", self.old_lic_code).as_str()) {
            Some(&publisher) => Some(publisher),
            None => None,
        }
    }

    // Function to lookup cart type
    fn cart_type_lookup(&self) -> Option<&'static str> {
        // Format the cart_type as a two-digit hexadecimal string
        let key = self.cart_type;
        // Use the key to look up in the HashMap
        match ROM_TYPES.get(&key) {
            Some(&cart_type) => Some(cart_type),
            None => None,
        }
    }

    fn old_license_lookup(&self) -> Option<&'static str> {
        match OLD_LICENSEE_CODES.get(&format!("{:02X}", self.old_lic_code).as_str()) {
            Some(&publisher) => Some(publisher),
            None => None,
        }
    }
}

lazy_static! {
    static ref NEW_LICENSEE_CODES: HashMap<&'static str, &'static str> = {
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
        map.insert("96", "Yonezawa/s'pal");
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

lazy_static! {
    static ref OLD_LICENSEE_CODES: HashMap<&'static str, &'static str> = {
        let mut map = HashMap::new();
        map.insert("00", "None");
        map.insert("01", "Nintendo");
        map.insert("08", "Capcom");
        map.insert("09", "HOT-B");
        map.insert("0A", "Jaleco");
        map.insert("0B", "Coconuts Japan");
        map.insert("0C", "Elite Systems");
        map.insert("13", "EA (Electronic Arts)");
        map.insert("18", "Hudson Soft");
        map.insert("19", "ITC Entertainment");
        map.insert("1A", "Yanoman");
        map.insert("1D", "Japan Clary");
        map.insert("1F", "Virgin Games Ltd.");
        map.insert("24", "PCM Complete");
        map.insert("25", "San-X");
        map.insert("28", "Kemco");
        map.insert("29", "SETA Corporation");
        map.insert("30", "Infogrames");
        map.insert("31", "Nintendo");
        map.insert("32", "Bandai");
        map.insert("33", "Use New Code");
        map.insert("34", "Konami");
        map.insert("35", "HectorSoft");
        map.insert("38", "Capcom");
        map.insert("39", "Banpresto");
        map.insert("3C", "Entertainment Interactive (stub)");
        map.insert("3E", "Gremlin");
        map.insert("41", "Ubi Soft");
        map.insert("42", "Atlus");
        map.insert("44", "Malibu Interactive");
        map.insert("46", "Angel");
        map.insert("47", "Spectrum HoloByte");
        map.insert("49", "Irem");
        map.insert("4A", "Virgin Games Ltd.");
        map.insert("4D", "Malibu Interactive");
        map.insert("4F", "U.S. Gold");
        map.insert("50", "Absolute");
        map.insert("51", "Acclaim Entertainment");
        map.insert("52", "Activision");
        map.insert("53", "Sammy USA Corporation");
        map.insert("54", "GameTek");
        map.insert("55", "Park Place");
        map.insert("56", "LJN");
        map.insert("57", "Matchbox");
        map.insert("59", "Milton Bradley Company");
        map.insert("5A", "Mindscape");
        map.insert("5B", "Romstar");
        map.insert("5C", "Naxat Soft");
        map.insert("5D", "Tradewest");
        map.insert("60", "Titus Interactive");
        map.insert("61", "Virgin Games Ltd.");
        map.insert("67", "Ocean Software");
        map.insert("69", "EA (Electronic Arts)");
        map.insert("6E", "Elite Systems");
        map.insert("6F", "Electro Brain");
        map.insert("70", "Infogrames");
        map.insert("71", "Interplay Entertainment");
        map.insert("72", "Broderbund");
        map.insert("73", "Sculptured Software");
        map.insert("75", "The Sales Curve Limited");
        map.insert("78", "THQ");
        map.insert("79", "Accolade");
        map.insert("7A", "Triffix Entertainment");
        map.insert("7C", "MicroProse");
        map.insert("7F", "Kemco");
        map.insert("80", "Misawa Entertainment");
        map.insert("83", "LOZC G.");
        map.insert("86", "Tokuma Shoten");
        map.insert("8B", "Bullet-Proof Software");
        map.insert("8C", "Vic Tokai Corp.");
        map.insert("8E", "Ape Inc.");
        map.insert("8F", "I'Max");
        map.insert("91", "Chunsoft Co.");
        map.insert("92", "Video System");
        map.insert("93", "Tsubaraya Productions");
        map.insert("95", "Varie");
        map.insert("96", "Yonezawa/S'Pal");
        map.insert("97", "Kemco");
        map.insert("99", "Arc");
        map.insert("9A", "Nihon Bussan");
        map.insert("9B", "Tecmo");
        map.insert("9C", "Imagineer");
        map.insert("9D", "Banpresto");
        map.insert("9F", "Nova");
        map.insert("A1", "Hori Electric");
        map.insert("A2", "Bandai");
        map.insert("A4", "Konami");
        map.insert("A6", "Kawada");
        map.insert("A7", "Takara");
        map.insert("A9", "Technos Japan");
        map.insert("AA", "Broderbund");
        map.insert("AC", "Toei Animation");
        map.insert("AD", "Toho");
        map.insert("AF", "Namco");
        map.insert("B0", "Acclaim Entertainment");
        map.insert("B1", "ASCII Corporation or Nexsoft");
        map.insert("B2", "Bandai");
        map.insert("B4", "Square Enix");
        map.insert("B6", "HAL Laboratory");
        map.insert("B7", "SNK");
        map.insert("B9", "Pony Canyon");
        map.insert("BA", "Culture Brain");
        map.insert("BB", "Sunsoft");
        map.insert("BD", "Sony Imagesoft");
        map.insert("BF", "Sammy Corporation");
        map.insert("C0", "Taito");
        map.insert("C2", "Kemco");
        map.insert("C3", "Square");
        map.insert("C4", "Tokuma Shoten");
        map.insert("C5", "Data East");
        map.insert("C6", "Tonkin House");
        map.insert("C8", "Koei");
        map.insert("C9", "UFL");
        map.insert("CA", "Ultra Games");
        map.insert("CB", "VAP, Inc.");
        map.insert("CC", "Use Corporation");
        map.insert("CD", "Meldac");
        map.insert("CE", "Pony Canyon");
        map.insert("CF", "Angel");
        map.insert("D0", "Taito");
        map.insert("D1", "SOFEL (Software Engineering Lab)");
        map.insert("D2", "Quest");
        map.insert("D3", "Sigma Enterprises");
        map.insert("D4", "ASK Kodansha Co.");
        map.insert("D6", "Naxat Soft");
        map.insert("D7", "Copya System");
        map.insert("D9", "Banpresto");
        map.insert("DA", "Tomy");
        map.insert("DB", "LJN");
        map.insert("DD", "Nippon Computer Systems");
        map.insert("DE", "Human Ent.");
        map.insert("DF", "Altron");
        map.insert("E0", "Jaleco");
        map.insert("E1", "Towa Chiki");
        map.insert("E2", "Yutaka");
        map.insert("E3", "Varie");
        map.insert("E5", "Epoch");
        map.insert("E7", "Athena");
        map.insert("E8", "Asmik Ace Entertainment");
        map.insert("E9", "Natsume");
        map.insert("EA", "King Records");
        map.insert("EB", "Atlus");
        map.insert("EC", "Epic/Sony Records");
        map.insert("EE", "IGS");
        map.insert("F0", "A Wave");
        map.insert("F3", "Extreme Entertainment");
        map.insert("FF", "LJN");
        map
    };
}
