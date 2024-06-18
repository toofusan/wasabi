extern crate alloc;

use crate::boot_info::BootInfo;
use crate::cmd;
use crate::efi::fs::EfiFileName;
use crate::error;
use crate::error::Error;
use crate::executor::yield_execution;
use crate::executor::Executor;
use crate::executor::Task;
use crate::executor::TimeoutFuture;
use crate::info;
use crate::mutex::Mutex;
use crate::print;
use crate::serial::SerialPort;
use alloc::collections::VecDeque;
use alloc::rc::Rc;
use alloc::string::String;
use core::str::FromStr;
use noli::bitmap::bitmap_draw_point;
use noli::bitmap::bitmap_draw_rect;
use noli::bitmap::BitmapBuffer;
use sabi::MouseEvent;

#[derive(Debug, PartialEq, Eq)]
pub enum KeyEvent {
    None,
    Char(char),
    Enter,
}

impl KeyEvent {
    pub fn to_char(&self) -> Option<char> {
        match self {
            KeyEvent::Char(c) => Some(*c),
            KeyEvent::Enter => Some('\n'),
            _ => None,
        }
    }
}

pub struct InputManager {
    input_queue: Mutex<VecDeque<char>>,
    cursor_queue: Mutex<VecDeque<MouseEvent>>,
}
impl InputManager {
    fn new() -> Self {
        Self {
            input_queue: Mutex::new(VecDeque::new(), "InputManager.input_queue"),
            cursor_queue: Mutex::new(VecDeque::new(), "InputManager.cursor_queue"),
        }
    }
    pub fn take() -> Rc<Self> {
        let mut instance = INPUT_MANAGER.lock();
        let instance = instance.get_or_insert_with(|| Rc::new(Self::new()));
        instance.clone()
    }
    pub fn push_input(&self, value: char) {
        self.input_queue.lock().push_back(value)
    }
    pub fn pop_input(&self) -> Option<char> {
        self.input_queue.lock().pop_front()
    }

    // x, y: 0f32..1f32, top left origin
    pub fn push_cursor_input_absolute(&self, e: MouseEvent) {
        self.cursor_queue.lock().push_back(e)
    }
    pub fn pop_cursor_input_absolute(&self) -> Option<MouseEvent> {
        self.cursor_queue.lock().pop_front()
    }
}
static INPUT_MANAGER: Mutex<Option<Rc<InputManager>>> = Mutex::new(None, "INPUT_MANAGER");

pub fn enqueue_input_tasks(executor: &mut Executor) {
    let serial_task = async {
        let sp = SerialPort::default();
        loop {
            if let Some(c) = sp.try_read() {
                if let Some(c) = char::from_u32(c as u32) {
                    let c = if c == '\r' { '\n' } else { c };
                    InputManager::take().push_input(c);
                }
            }
            TimeoutFuture::new_ms(20).await;
            yield_execution().await;
        }
    };
    let init_task = async {
        info!("running init");
        let boot_info = BootInfo::take();
        let root_files = boot_info.root_files();
        let root_files: alloc::vec::Vec<&crate::boot_info::File> =
            root_files.iter().filter_map(|e| e.as_ref()).collect();
        let init_txt = EfiFileName::from_str("init.txt")?;
        let init_txt = root_files
            .iter()
            .find(|&e| e.name() == &init_txt)
            .ok_or(Error::Failed("init.txt not found"))?;
        let init_txt = String::from_utf8_lossy(init_txt.data());
        for line in init_txt.trim().split('\n') {
            if let Err(e) = cmd::run(line).await {
                error!("{e:?}");
            };
        }
        Ok(())
    };
    let console_task = async {
        info!("console_task has started");
        let mut s = String::new();
        loop {
            if let Some(c) = InputManager::take().pop_input() {
                if c == '\r' || c == '\n' {
                    if let Err(e) = cmd::run(&s).await {
                        error!("{e:?}");
                    };
                    s.clear();
                }
                match c {
                    'a'..='z' | 'A'..='Z' | '0'..='9' | ' ' | '.' => {
                        print!("{c}");
                        s.push(c);
                    }
                    '\x7f' | '\x08' => {
                        print!("{0} {0}", 0x08 as char);
                        s.pop();
                    }
                    _ => {
                        // Do nothing
                    }
                }
            }
            TimeoutFuture::new_ms(20).await;
            yield_execution().await;
        }
    };
    let mouse_cursor_task = async {
        const CURSOR_SIZE: i64 = 16;
        let mut cursor_bitmap = BitmapBuffer::new(CURSOR_SIZE, CURSOR_SIZE, CURSOR_SIZE);
        for y in 0..CURSOR_SIZE {
            for x in 0..(CURSOR_SIZE - y) {
                if x <= y {
                    bitmap_draw_point(&mut cursor_bitmap, 0x00ff00, x, y)
                        .expect("Failed to paint cursor");
                }
            }
        }
        let mut vram = BootInfo::take().vram();

        noli::bitmap::draw_bmp_clipped(&mut vram, &cursor_bitmap, 100, 100)
            .ok_or(Error::Failed("Failed to draw mouse cursor"))?;

        loop {
            if let Some(MouseEvent {
                position: p,
                button: b,
            }) = InputManager::take().pop_cursor_input_absolute()
            {
                let color = (b.l() as u32) * 0xff0000;
                let color = !color;

                bitmap_draw_rect(&mut vram, color, p.x, p.y, 1, 1)?;
                /*
                crate::graphics::draw_bmp_clipped(&mut vram, &cursor_bitmap, p.x, p.y)
                    .ok_or(Error::Failed("Failed to draw mouse cursor"))?;
                */
            }
            TimeoutFuture::new_ms(15).await;
            yield_execution().await;
        }
    };
    executor.spawn(Task::new(serial_task));
    executor.spawn(Task::new(console_task));
    executor.spawn(Task::new(mouse_cursor_task));
    executor.spawn(Task::new(init_task));
}
