extern crate encoding_rs;

use encoding_rs::WINDOWS_1251;

mod sub_proc;

use sub_proc::SubProc;

fn cmd_test() {
    let mut sql_proc = SubProc::new();
    sql_proc.run();

    sql_proc.push(b"conn user/pass@tns\r\n");
    sql_proc.push(b"select 1 from dual;\r\n");

    loop {

        let mut resp_collect = Vec::<u8>::new();

        for b in sql_proc.packets() {
            resp_collect.push(b);
        }

        let (cow, _encoding_used, _had_errors)
        = WINDOWS_1251.decode(&resp_collect);

        let std_msg = String::from(cow);

        std::thread::sleep(std::time::Duration::from_millis(300));

        print!("{}", &std_msg);
    }
}


fn main() {
    cmd_test();

    println!("Hello, world!");
}
