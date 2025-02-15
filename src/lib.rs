mod models;
mod windows_storage;

pub use models::*;
pub use windows_storage::*;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        let disks = get_disks().unwrap();

        println!("Disks:");
        for disk in disks {
            println!("{:?}", disk);
        }
    }
}
