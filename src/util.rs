use esp_idf_svc::sys::{
    MALLOC_CAP_DEFAULT,
    MALLOC_CAP_INTERNAL,
    MALLOC_CAP_SPIRAM,
    heap_caps_get_free_size,
};


pub fn show_memory_status() {
    let internal = unsafe { heap_caps_get_free_size(MALLOC_CAP_INTERNAL) };
    // SPI-attached PSRAM
    let external = unsafe { heap_caps_get_free_size(MALLOC_CAP_SPIRAM) };
    let total = unsafe { heap_caps_get_free_size(MALLOC_CAP_DEFAULT) };
    println!("Free heap mem: IRAM {} + PSRAM {} = {} bytes", internal, external, total);
}
