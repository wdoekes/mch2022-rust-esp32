use std::sync::{Arc, Mutex};

use esp_idf_svc::hal::i2c::I2cDriver;
use esp_idf_svc::hal::delay::TICK_RATE_HZ;


#[derive(Debug, thiserror::Error)]
#[error("unsupported firmware version: {0:#X}")]
pub struct UnsupportedFirmware(u8);

pub type SharedI2c<'a> = Arc<Mutex<I2cDriver<'a>>>;

const DEPRECATED_TIMEOUT: u32 = TICK_RATE_HZ; // exactly 1 second
const RP2040_I2C_ADDR: u8 = 0x17;
//const GPIO_INT_RP2040: u8 = 34;

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
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
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

//enum { RP2040_BL_REG_FW_VER, RP2040_BL_REG_BL_VER, RP2040_BL_REG_BL_STATE, RP2040_BL_REG_BL_CTRL };


pub struct Rp2040<'a> {
    i2c: SharedI2c<'a>,
    addr: u8,
    fw_version: u8,
}

impl<'a> Rp2040<'a> {
    pub fn new(i2c: SharedI2c<'a>) -> Self {
        Self { i2c, addr: RP2040_I2C_ADDR, fw_version: 0 }
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
        Ok((buf[1] as u16) << 8 | (buf[0] as u16))
    }

    pub fn read_vbat(&mut self) -> anyhow::Result<f32> {
        const CONVERSION_FACTOR: f32 = 3.3_f32 / ((1 << 12) as f32);  // 12-bit ADC with 3.3v vref
        let raw = self.read_vbat_raw()?;
        Ok((raw as f32) * CONVERSION_FACTOR * 2.0_f32) // Connected through 100k/100k divider
    }

    fn read_reg(&self, reg: Rp2040Reg, buf: &mut [u8]) -> anyhow::Result<()> {
        self.i2c.lock().unwrap().write_read(self.addr, &[reg.into()], buf, DEPRECATED_TIMEOUT)?;
        Ok(())
    }

    /*
    fn write_reg(&self, reg: u8, data: &[u8]) -> anyhow::Result<()> {
        let mut out = Vec::with_capacity(1 + data.len());
        out.push(reg);
        out.extend_from_slice(data);
        self.i2c.lock().unwrap().write(self.addr, &out, DEPRECATED_TIMEOUT)?;
        Ok(())
    }
    */
}

/*
typedef void (*rp2040_intr_t)();

typedef struct {
    int              i2c_bus;
    int              i2c_address;
    int              pin_interrupt;
    xQueueHandle     queue;
    xSemaphoreHandle i2c_semaphore;
    rp2040_intr_t    _intr_handler;
    TaskHandle_t     _intr_task_handle;
    xSemaphoreHandle _intr_trigger;
    uint8_t          _gpio_direction;
    uint8_t          _gpio_value;
    uint8_t          _fw_version;
} RP2040;

typedef struct _rp2040_input_message {
    uint8_t input;
    bool    state;
} rp2040_input_message_t;

esp_err_t rp2040_init(RP2040* device);

esp_err_t rp2040_get_firmware_version(RP2040* device, uint8_t* version);
*/
