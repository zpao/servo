use core::pipes::{Port, Chan};
use content::content_task::{ControlMsg, Timer, ExitMsg};
use js::jsapi::JSVal;
use dvec::DVec;
use util::task::spawn_listener;
use std::timer;
use std::uv_global_loop;

pub enum TimerControlMsg {
    TimerMessage_Fire(~TimerData),
    TimerMessage_Close,
    TimerMessage_TriggerExit //XXXjdm this is just a quick hack to talk to the content task
}

pub struct Window {
    timer_chan: Chan<TimerControlMsg>,

    drop {
        self.timer_chan.send(TimerMessage_Close);
    }
}

// Holder for the various JS values associated with setTimeout
// (ie. function value to invoke and all arguments to pass
//      to the function when calling it)
pub struct TimerData {
    funval: JSVal,
    args: DVec<JSVal>,
}

pub fn TimerData(argc: libc::c_uint, argv: *JSVal) -> TimerData {
    unsafe {
        let data = TimerData {
            funval : *argv,
            args : DVec(),
        };

        let mut i = 2;
        while i < argc as uint {
            data.args.push(*ptr::offset(argv, i));
            i += 1;
        };

        data
    }
}

// FIXME: delayed_send shouldn't require Copy
#[allow(non_implicitly_copyable_typarams)]
impl Window {
    fn alert(s: &str) {
        // Right now, just print to the console
        io::println(fmt!("ALERT: %s", s));
    }

    fn close() {
        self.timer_chan.send(TimerMessage_TriggerExit);
    }

    fn setTimeout(&self, timeout: int, argc: libc::c_uint, argv: *JSVal) {
        let timeout = int::max(0, timeout) as uint;

        // Post a delayed message to the per-window timer task; it will dispatch it
        // to the relevant content handler that will deal with it.
        timer::delayed_send(&uv_global_loop::get(),
                            timeout,
                            &self.timer_chan,
                            TimerMessage_Fire(~TimerData(argc, argv)));
    }
}

pub fn Window(content_chan: pipes::SharedChan<ControlMsg>) -> Window {
        
    Window {
        timer_chan: do spawn_listener |timer_port: Port<TimerControlMsg>| {
            loop {
                match timer_port.recv() {
                    TimerMessage_Close => break,
                    TimerMessage_Fire(td) => {
                        content_chan.send(Timer(td));
                    }
                    TimerMessage_TriggerExit => content_chan.send(ExitMsg)
                }
            }
        }
    }
}
