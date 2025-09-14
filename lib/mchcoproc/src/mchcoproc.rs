use std::num::NonZero;
use std::sync::{Arc, Mutex, mpsc};
use std::thread;

use esp_idf_svc::hal::i2c::I2cDriver;
use esp_idf_svc::hal::delay::TICK_RATE_HZ;
use esp_idf_svc::hal::gpio::{Input, InputPin, InterruptType, Pin, PinDriver};
use esp_idf_svc::hal::task::notification;

use strum::FromRepr;


const DEPRECATED_TIMEOUT: u32 = TICK_RATE_HZ; // exactly 1 second
const RP2040_I2C_ADDR: u8 = 0x17;
//FIXME: nice function that returns peripherals.gpio34?
//const GPIO_INT_RP2040: u8 = 34;
//enum { RP2040_BL_REG_FW_VER, RP2040_BL_REG_BL_VER,
//       RP2040_BL_REG_BL_STATE, RP2040_BL_REG_BL_CTRL };


#[derive(Debug, thiserror::Error)]
#[error("unsupported firmware version: {0:#X}")]
pub struct UnsupportedFirmware(u8);

pub type SharedI2c<'a> = Arc<Mutex<I2cDriver<'a>>>;

#[repr(u8)]
#[derive(Copy, Clone, Debug)]
pub enum Rp2040Reg {
    FwVer = 0,
    GpioDir,
    GpioIn,
    GpioOut,
    LcdBacklight,
    Fpga,
    Input1,
    Input2,
    Interrupt1,
    Interrupt2,
    AdcTrigger,
    AdcValueVusbLo,
    AdcValueVusbHi,
    AdcValueVbatLo,
    AdcValueVbatHi,
    Usb,
    BlTrigger,
    WebusbMode,
    CrashDebug,
    ResetLock,
    ResetAttempted,
    ChargingState,
    AdcValueTempLo,
    AdcValueTempHi,
    Uid0, // unique board identifier of the rp2040
    Uid1,
    Uid2,
    Uid3,
    Uid4,
    Uid5,
    Uid6,
    Uid7,
    Scratch0,  // used by the esp32 to store boot parameters, can also be read and written to from webusb
    Scratch1,
    Scratch2,
    Scratch3,
    Scratch4,
    Scratch5,
    Scratch6,
    Scratch7,
    Scratch8,
    Scratch9,
    Scratch10,
    Scratch11,
    Scratch12,
    Scratch13,
    Scratch14,
    Scratch15,
    Scratch16,
    Scratch17,
    Scratch18,
    Scratch19,
    Scratch20,
    Scratch21,
    Scratch22,
    Scratch23,
    Scratch24,
    Scratch25,
    Scratch26,
    Scratch27,
    Scratch28,
    Scratch29,
    Scratch30,
    Scratch31,
    Scratch32,
    Scratch33,
    Scratch34,
    Scratch35,
    Scratch36,
    Scratch37,
    Scratch38,
    Scratch39,
    Scratch40,
    Scratch41,
    Scratch42,
    Scratch43,
    Scratch44,
    Scratch45,
    Scratch46,
    Scratch47,
    Scratch48,
    Scratch49,
    Scratch50,
    Scratch51,
    Scratch52,
    Scratch53,
    Scratch54,
    Scratch55,
    Scratch56,
    Scratch57,
    Scratch58,
    Scratch59,
    Scratch60,
    Scratch61,
    Scratch62,
    Scratch63,
    IrAddressLo,
    IrAddressHi,
    IrCommand,
    IrTrigger,
    Reserved11,
    Reserved12,
    Reserved13,
    Reserved14,
    Ws2812Mode,
    Ws2812Trigger,
    Ws2812Length,
    Ws2812Speed,
    Ws2812Led0Data0,
    Ws2812Led0Data1,
    Ws2812Led0Data2,
    Ws2812Led0Data3,
    Ws2812Led1Data0,
    Ws2812Led1Data1,
    Ws2812Led1Data2,
    Ws2812Led1Data3,
    Ws2812Led2Data0,
    Ws2812Led2Data1,
    Ws2812Led2Data2,
    Ws2812Led2Data3,
    Ws2812Led3Data0,
    Ws2812Led3Data1,
    Ws2812Led3Data2,
    Ws2812Led3Data3,
    Ws2812Led4Data0,
    Ws2812Led4Data1,
    Ws2812Led4Data2,
    Ws2812Led4Data3,
    Ws2812Led5Data0,
    Ws2812Led5Data1,
    Ws2812Led5Data2,
    Ws2812Led5Data3,
    Ws2812Led6Data0,
    Ws2812Led6Data1,
    Ws2812Led6Data2,
    Ws2812Led6Data3,
    Ws2812Led7Data0,
    Ws2812Led7Data1,
    Ws2812Led7Data2,
    Ws2812Led7Data3,
    Ws2812Led8Data0,
    Ws2812Led8Data1,
    Ws2812Led8Data2,
    Ws2812Led8Data3,
    Ws2812Led9Data0,
    Ws2812Led9Data1,
    Ws2812Led9Data2,
    Ws2812Led9Data3,
    MscControl,
    MscState,
    Msc0BlockCountLoA,
    Msc0BlockCountLoB,
    Msc0BlockCountLoC,
    Msc0BlockCountHi,
    Msc0BlockSizeLo,
    Msc0BlockSizeHi,
    Msc1BlockCountLoA,
    Msc1BlockCountLoB,
    Msc1BlockCountLoC,
    Msc1BlockCountHi,
    Msc1BlockSizeLo,
    Msc1BlockSizeHi,
}

impl From<Rp2040Reg> for u8 {
    fn from(r: Rp2040Reg) -> Self { r as u8 }
}


#[repr(u8)]
#[derive(Copy, Clone, Debug, Eq, FromRepr, PartialEq)]
pub enum Rp2040Input {
    ButtonHome = 0,
    ButtonMenu,
    ButtonStart,
    ButtonAccept,
    ButtonBack,
    FpgaCdone,
    BatteryCharging,
    ButtonSelect,
    JoystickLeft,
    JoystickPress,
    JoystickDown,
    JoystickUp,
    JoystickRight,
}

impl From<Rp2040Input> for u8 {
    fn from(i: Rp2040Input) -> Self { i as u8 }
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub struct Rp2040InputEvent {
    pub input: Rp2040Input,
    pub is_released: bool,
}

impl Rp2040InputEvent {
    pub fn new(input: Rp2040Input, is_released: bool) -> Self {
        Self { input, is_released }
    }
}
pub struct Rp2040 {
    i2c: SharedI2c<'static>,
    addr: u8,
    fw_version: u8,
    //gpio_dir_bits: u8, // direction (in/out)
    //gpio_val_bits: u8, // value (off/on)
}

pub type SharedRp2040 = Arc<Mutex<Rp2040>>;


impl Rp2040 {
    pub fn new(i2c: SharedI2c<'static>) -> Self {
        Self {
            i2c,
            addr: RP2040_I2C_ADDR,
            fw_version: 0,
            //gpio_dir_bits: 0,
            //gpio_val_bits: 0,
        }
    }

    pub fn setup_interrupt<PIN: Pin + InputPin>(
        mut self,
        pin: PIN,
        event_sender: mpsc::Sender<Rp2040InputEvent>,
    ) -> anyhow::Result<SharedRp2040> {
        self.fw_version = self.get_firmware_version()?;
        if self.fw_version < 1 {
            anyhow::bail!("Unsupported FW version {}", self.fw_version);
        }

        //let mut dir = [0u8];
        //self.read_reg(Rp2040Reg::GpioDir, &mut dir)?;
        //self.gpio_dir_bits = dir[0];

        //let mut val = [0u8];
        //self.read_reg(Rp2040Reg::GpioOut, &mut val)?;
        //self.gpio_val_bits = val[0];

        let rp2040 = Arc::new(Mutex::new(self));
        setup_interrupt_and_task(pin, event_sender, rp2040.clone())?;
        Ok(rp2040)
    }

    pub fn get_firmware_version(&mut self) -> anyhow::Result<u8> {
        let mut buf = [0u8; 1];
        self.read_reg(Rp2040Reg::FwVer, &mut buf)?;
        self.fw_version = buf[0];
        Ok(self.fw_version)
    }

    fn read_vbat_raw(&mut self) -> anyhow::Result<u16> {
        if (self.fw_version < 0x02) || (self.fw_version == 0xFF) {
            return Err(UnsupportedFirmware(self.fw_version).into());
        }
        let mut buf = [0u8; 2];
        self.read_reg(Rp2040Reg::AdcValueVbatLo, &mut buf)?;
        Ok(((buf[1] as u16) << 8) | (buf[0] as u16))
    }

    pub fn read_vbat(&mut self) -> anyhow::Result<f32> {
        // 12-bit ADC with 3.3v vref
        const CONVERSION_FACTOR: f32 = 3.3_f32 / ((1 << 12) as f32);
        let raw = self.read_vbat_raw()?;
        // Connected through 100k/100k divider
        Ok((raw as f32) * CONVERSION_FACTOR * 2.0_f32)
    }

    // These should be read by the local task, which is triggered by the
    // interrupt. Others won't need to be reading this.
    fn read_inputs(&mut self) -> anyhow::Result<Vec<Rp2040InputEvent>> {
        let mut buf = [0u8; 4];
        self.read_reg(Rp2040Reg::Input1, &mut buf)?;
        let state = u32::from_le_bytes(buf);
        let interrupt = (state >> 16) as u16;
        let values = (state & 0xffff) as u16;

        let mut events = Vec::new();

        for idx in 0..16 {
            if ((interrupt >> idx) & 0x1) != 0 {
                if let Some(input) = Rp2040Input::from_repr(idx) {
                    let is_released = ((values >> idx) & 0x1) == 0;
                    events.push(Rp2040InputEvent::new(input, is_released));
                }
            }
        }

        Ok(events)
    }

    fn read_reg(&self, reg: Rp2040Reg, buf: &mut [u8]) -> anyhow::Result<()> {
        self.i2c.lock().unwrap()
            .write_read(self.addr, &[reg.into()], buf, DEPRECATED_TIMEOUT)?;
        Ok(())
    }

    // NOTE: Writing RC5 infrared requires modified RP2040 firmware.
    pub fn write_ir_trigger_rc5(
        &self, toggle: bool,
        address: u16,
        command: u16,
    ) {
        // IrTrigger 0x2 = RC5, and 0x3 is RC5 with toggle.
        let ir_proto: u8 = if toggle { 0x2 } else { 0x3 };
        let buf: [u8; 4] = [
            (address & 0xff) as u8, // IrAddressLo
            (address >> 8) as u8,   // IrAddressHi
            (command & 0xff) as u8, // IrCommand
            ir_proto,
        ];
        self.write_reg(Rp2040Reg::IrAddressLo, &buf).unwrap(); // XXX: unwrap
    }

    fn write_reg(&self, reg: Rp2040Reg, data: &[u8]) -> anyhow::Result<()> {
        let mut out = Vec::with_capacity(1 + data.len());
        out.push(reg.into());
        out.extend_from_slice(data);
        log::info!("write_reg: {:?}", out);
        self.i2c.lock().unwrap().write(self.addr, &out, DEPRECATED_TIMEOUT)?;
        Ok(())
    }
}


fn setup_interrupt_and_task<PIN: Pin + InputPin>(
    interrupt_pin: PIN,
    event_sender: mpsc::Sender<Rp2040InputEvent>,
    rp2040: Arc<Mutex<Rp2040>>,
) -> anyhow::Result<()> {
    // This logs:
    // I ... gpio: GPIO[34]| InputEn: 0| OutputEn: 0| OpenDrain: 0| Pullup: 0..
    let mut intpin = PinDriver::input(interrupt_pin)?;

    #[cfg(feature = "unused-force-pullup")] // intpin.set_pull(Pull::Up)
    force_gpio_set_pull(&mut intpin, esp_idf_svc::hal::gpio::Pull::Up);

    intpin.set_interrupt_type(InterruptType::NegEdge)?;

    // Send task handle back from the spawned thread/task.
    let (notifier_tx, notifier_rx) = mpsc::channel();
    // Send the pindriver into the task.
    let (intpin_tx, intpin_rx) =
        mpsc::channel::<PinDriver<'static, PIN, Input>>();

    // Task: block until ISR notifies
    log::info!("PRESPAWN: ISR should be up...");
    thread::spawn(move || {
        let notification = notification::Notification::new();
        notifier_tx.send(notification.notifier()).unwrap();

        // IMPORTANT! Don't drop the intpin, as the driver would reset it.
        // Luckily we need it because we need to enable_interrupt() on it
        // after every action.
        let mut intpin = intpin_rx.recv().unwrap();

        loop {
            // After device boot, we first need to clear all events:
            // - Rp2040InputEvent { input: FpgaCdone, is_released: false }
            log::info!("SPAWN: Read me some events...");

            // Get all events and drop the rp2040 lock immediately.
            // (Generally 1 after ISR poke, 1 at boot, or 0 after
            // firmware restart.)
            let events = {
                let mut rp = rp2040.lock().unwrap();
                rp.read_inputs().ok().map(|v| v.to_vec()).unwrap_or_default()
            };

            for ev in events {
                log::info!("Got event: {:?}", ev);
                if let Err(err) = event_sender.send(ev) {
                    log::warn!("Could not send event: {:?} - {}", ev, err);
                }
            }

            // IMPORTANT! Run after every handle.
            // (If this fails, we cannot do buttons anymore. Might as
            // well die.)
            intpin.enable_interrupt().unwrap();

            log::info!("SPAWN: Waiting for ISR...");
            notification.wait_any();
        }
    });

    // The thread/task knows it's "handle". We need that to notify it.
    let notifier = notifier_rx.recv().unwrap();

    unsafe {
        // This must be fast. Supposedly we should use
        // intpin.subscribe_nonstatic() because notifier could
        // otherwise go out of scope, but it appears to work
        // with intpin.subscribe() too.
        intpin.subscribe(move || {
            notifier.notify_and_yield(NonZero::new(1).unwrap());
        })?;
    }

    // We're done with intpin, but our task is not. Move it there.
    intpin_tx.send(intpin).unwrap();

    Ok(())
}


#[cfg(feature = "unused-force-pullup")]
// > GPIOs 34 to 39 are GPIs - input only pins. These pins don't have
// > internal pull-ups or pull-down resistors. They can't be used as
// > outputs, so use these pins only as inputs.
//
// Supposedly.
//
// So, GPIO34 does not implement OutputPin. And then intpin.set_pull(Pull::Up)
// is not implemented. Instead we force it anyway, as otherwise things don't
// work.
//
// This seemed to be the case, but after further testing, we can do without.
// Leave the code behind a feature toggle for now.
fn force_gpio_set_pull<PIN: Pin + InputPin>(
    pindriver: &mut PinDriver<'static, PIN, Input>,
    mode: esp_idf_svc::hal::gpio::Pull,
) {
    match mode {
        esp_idf_svc::hal::gpio::Pull::Floating => unsafe {
            esp_idf_svc::hal::sys::gpio_pulldown_dis(pindriver.pin());
            esp_idf_svc::hal::sys::gpio_pullup_dis(pindriver.pin());
        },
        esp_idf_svc::hal::gpio::Pull::Down => unsafe {
            esp_idf_svc::hal::sys::gpio_pulldown_en(pindriver.pin());
            esp_idf_svc::hal::sys::gpio_pullup_dis(pindriver.pin());
        },
        esp_idf_svc::hal::gpio::Pull::Up => unsafe {
            esp_idf_svc::hal::sys::gpio_pullup_en(pindriver.pin());
            esp_idf_svc::hal::sys::gpio_pulldown_dis(pindriver.pin());
        },
        esp_idf_svc::hal::gpio::Pull::UpDown => unsafe {
            esp_idf_svc::hal::sys::gpio_pulldown_en(pindriver.pin());
            esp_idf_svc::hal::sys::gpio_pullup_en(pindriver.pin());
        },
    }
}
