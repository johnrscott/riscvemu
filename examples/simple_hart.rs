use riscvemu::hart::Hart;

fn main() {
    let mut hart = Hart::default();
    println!("{:#?}", hart);

    match hart.step() {
        Ok(_) => println!("Done"),
        Err(trap) => println!("{trap}"),
    }
}
