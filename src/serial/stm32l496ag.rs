//! Serial devices on the STM32L496AG

use stm32l4x6::{USART1, USART2, USART3, UART4, UART5};

use gpio::stm32l496ag::gpio::{PA9, PA10, PA2, PA3, PA0, PA1, PA15};
use gpio::stm32l496ag::gpio::{PB6, PB7, PB10, PB11};
use gpio::stm32l496ag::gpio::{PC4, PC10, PC5, PC11, PC12};
use gpio::stm32l496ag::gpio::{PD5, PD6, PD8, PD9, PD2};
use gpio::stm32l496ag::gpio::{PG9, PG10};
use gpio::stm32l496ag::gpio::{AF3, AF7, AF8};

use super::*;

unsafe impl TxPin<USART1> for PA9<AF7> {}
unsafe impl TxPin<USART1> for PB6<AF7> {}
unsafe impl TxPin<USART1> for PG9<AF7> {}

unsafe impl RxPin<USART1> for PA10<AF7> {}
unsafe impl RxPin<USART1> for PB7<AF7> {}
unsafe impl RxPin<USART1> for PG10<AF7> {}


unsafe impl TxPin<USART2> for PA2<AF7> {} // exposed via STM32L496 Discovery STLINK
unsafe impl TxPin<USART2> for PD5<AF7> {}

unsafe impl RxPin<USART2> for PA3<AF7> {}
unsafe impl RxPin<USART2> for PA15<AF3> {}
unsafe impl RxPin<USART2> for PD6<AF7> {} // exposed via STM32L496 Discovery STLINK


unsafe impl TxPin<USART3> for PB10<AF7> {}
unsafe impl TxPin<USART3> for PC4<AF7> {}
unsafe impl TxPin<USART3> for PC10<AF7> {}
unsafe impl TxPin<USART3> for PD8<AF7> {}

unsafe impl RxPin<USART3> for PB11<AF7> {}
unsafe impl RxPin<USART3> for PC5<AF7> {}
unsafe impl RxPin<USART3> for PC11<AF7> {}
unsafe impl RxPin<USART3> for PD9<AF7> {}


unsafe impl TxPin<UART4> for PA0<AF8> {}
unsafe impl TxPin<UART4> for PC10<AF8> {}

unsafe impl RxPin<UART4> for PA1<AF8> {}
unsafe impl RxPin<UART4> for PC11<AF8> {}


unsafe impl TxPin<UART5> for PC12<AF8> {}
unsafe impl RxPin<UART5> for PD2<AF8> {}
