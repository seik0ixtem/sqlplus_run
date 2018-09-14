
// использование этой директивы позволяет погасить консольное окно.
// но! после это, например, консольный логгер будет паниковать при создании.
#![windows_subsystem="windows"]

#[macro_use] extern crate log;
#[macro_use] extern crate sciter;

extern crate simplelog;
extern crate config;
extern crate chrono;
extern crate fs_extra;
extern crate regex;
extern crate encoding_rs;


mod sub_proc;

use sub_proc::SubProc;

use std::{fs, panic};
use std::fs::{OpenOptions};
use std::path::{Path};
use std::ffi::{OsStr};
use simplelog::*;
use encoding_rs::WINDOWS_1251;
use chrono::prelude::*;

// rustsym panics on this:
// use sciter::{Element, dom::event::*, dom::HELEMENT, value::Value};
// so i temporary expand it
use sciter::Element;
use sciter::dom::HELEMENT;
use sciter::value::Value;

// traits
use std::str::FromStr;
use std::io::{Write};

const PKG_NAME: &'static str = env!("CARGO_PKG_NAME");

//const DEFAULT_TERM_LOG_LEVEL: LevelFilter = LevelFilter::Debug;
const DEFAULT_FILE_LOG_LEVEL: LevelFilter = LevelFilter::Debug;

struct EventHandler<'a> {
//    settings: &'a config::Config,
    sqp: &'a mut SubProc,
    root: Option<Element>,
}

impl<'a> EventHandler<'a> {
//    fn dp_file_open(&self, args: &[Value], root: &Element) -> Option<Value> {
    fn dp_file_open(&mut self, fname: String, cb_done: sciter::Value) -> bool {

        let fname = fname;

        // нужна обрезать file:// в начале имени файла
        let install_file: String =
            fname.chars().skip("file://".len()).collect();

        let orig_template: String;

        let data =
            match fs::read_to_string(Path::new(&install_file)) {
                Ok(r) => {
                    info!("read_to_string seems ok!");
                    r
                }
                ,Err(ref err) if std::io::ErrorKind::InvalidData == err.kind() => {
                    info!("Обнаружили ошибку чтения индекс-файла. Пробуем\
                                конвертировать");
                    // сбросим файл на начало, потому что это типа поток.
                    let template_1251 =
                        fs::read(Path::new(&install_file)).unwrap();

                    let (cow, _encoding_used, had_errors)
                    = WINDOWS_1251.decode(&template_1251);

                    orig_template = String::from(cow);

                    info!("Были ошибки конвертации: {:?}", had_errors);
                    debug!("Наконвертировали: {:?}", &orig_template);
                    orig_template
                }
                ,Err(err) => panic!("Неожиданная ошибка: {:?}", err)
            };

        cb_done.call(None, &make_args!(data), None).unwrap();

        return true;
    }

    fn dp_sqp_send(&mut self, data: String) {
        self.sqp.push(data.as_bytes());
        self.sqpout_push(data);
    }

    fn sqpout_push(&mut self, data: String) {
        let root = self.root.as_ref().unwrap();

        root.call_function("sqpout_push", &make_args!(data)).unwrap();
    }

    fn dp_start_sqp_listen(&mut self, cb_got_data: sciter::Value) {
        let mut resp_collect = Vec::<u8>::new();

        for b in self.sqp.packets() {
            resp_collect.push(b);
        }

        let (cow, _encoding_used, _had_errors)
        = WINDOWS_1251.decode(&resp_collect);

        let std_msg = String::from(cow);

        cb_got_data.call(None, &make_args!(std_msg), None).unwrap();
    }
}

impl<'a> sciter::EventHandler for EventHandler<'a> {
    fn attached(&mut self, root: HELEMENT) {
        println!("attached to: {:?}", Element::from(root));
        self.root = Some(Element::from(root));
    }

    dispatch_script_call! {
        fn dp_file_open(String, Value);
        fn dp_sqp_send(String);
        fn dp_start_sqp_listen(Value);
    }
}


//
//fn cmd_test() {
//    let mut sql_proc = SubProc::new();
//    sql_proc.run();
//
//    sql_proc.push(b"conn user/pass@tns\r\n");
//    sql_proc.push(b"select 1 from dual;\r\n");
//
//    loop {
//
//        let mut resp_collect = Vec::<u8>::new();
//
//        for b in sql_proc.packets() {
//            resp_collect.push(b);
//        }
//
//        let (cow, _encoding_used, _had_errors)
//            = WINDOWS_1251.decode(&resp_collect);
//
//        let std_msg = String::from(cow);
//
//        std::thread::sleep(std::time::Duration::from_millis(300));
//
//        print!("{}", &std_msg);
//    }
//}
//

fn main() {
    // в оконном приложении паниковать дефолтно трудно.
    // паниковать в логи трудно, например, если логи ещё не сконфигурирован. Поэтому паниковать
    // будем в отдельный файл panic.log.

    let orig_panic = panic::take_hook();

    panic::set_hook(Box::new(move |e: &panic::PanicInfo| {
        // паники изнутри этого метода до нас никогда не долетят.
        let mut panic_file = OpenOptions::new()
            .append(true)
            .create(true)
            .open("panic.log")
            .unwrap();

        panic_file.write_fmt(format_args!("{} {:?}\r\n", Local::now(), &e)).unwrap();

        drop(panic_file);

        orig_panic(e);
    }));


    // чтение конфига из settings.toml
    let mut settings = config::Config::default();
    settings.merge(config::File::with_name("settings")).unwrap();

    CombinedLogger::init(
        vec![
            // искал какой-нибудь способ в настройки логгера передать числовое значение - не нашёл.
            // реализовывать свитч самому здесь - лень ваще.
            // Отключил консольный вывод, чтобы под вендой красиво работало.
            // TermLogger::new(
            //             LevelFilter::from_str(
            //                 settings.get::<String>("main.term_log_level").unwrap().as_str())
            //             .unwrap_or(DEFAULT_TERM_LOG_LEVEL)
            //         , Config::default())
            //     .expect("Failed to create TermLogger")
            // ,
            WriteLogger::new(
                //LevelFilter::from_str(sets.get("main.file_log_level").unwrap())
                LevelFilter::from_str(settings.get::<String>("main.file_log_level").unwrap().as_str())
                    .unwrap_or(DEFAULT_FILE_LOG_LEVEL)
                ,Config::default()
                ,OpenOptions::new()
                    .append(true)
                    .create(true)
                    .open(Path::new(OsStr::new(&format!("{}.log", PKG_NAME))))
                    .unwrap())]).expect("Failed to configure logging.");

    info!("Started. Version = {}", env!("CARGO_PKG_VERSION"));

    // 1. Подготавливаем структуры.
    // 2. Запускаем sql*plus
    // 3. Запускаем интерфейс

    // 1
    let mut sql_proc = SubProc::new();
    sql_proc.run();

    let html = include_bytes!("face.htm");
    let mut frame = sciter::Window::new();
    frame.load_html(html, Some("example://face.htm"));

    frame.event_handler(EventHandler {sqp: &mut sql_proc, root: None});
    frame.set_title(format!("{}: Нарезатель ({})", PKG_NAME, env!("CARGO_PKG_VERSION")).as_str());

    // после этого основной поток уйдёт в луп обслуживания графического приложения.
    frame.run_app();

}
