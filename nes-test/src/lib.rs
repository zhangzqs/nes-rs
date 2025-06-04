mod neslog;

#[cfg(test)]
mod tests {
    use std::io::BufRead;

    use nes_cartridge::NESFile;

    use crate::neslog::NESLog;

    use super::*;

    #[test]
    fn it_works() {
        let nes = nes_cartridge::NESFile::from_file("testfiles/nestest.nes");
        let cartridge = nes_cartridge::CartridgeImpl::new(nes);
        


        let testlogs = std::fs::read("testfiles/nestest.txt").unwrap();
        for line in testlogs.lines() {
            let log = NESLog::parse_line(&line.unwrap());
            println!("{:?}", log);
        }

    }
}
