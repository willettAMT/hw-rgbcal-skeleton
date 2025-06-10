(function() {
    var implementors = Object.fromEntries([["embassy_embedded_hal",[["impl&lt;'a, M, BUS, E&gt; <a class=\"trait\" href=\"embedded_hal/blocking/i2c/trait.Read.html\" title=\"trait embedded_hal::blocking::i2c::Read\">Read</a> for <a class=\"struct\" href=\"embassy_embedded_hal/shared_bus/blocking/i2c/struct.I2cDevice.html\" title=\"struct embassy_embedded_hal::shared_bus::blocking::i2c::I2cDevice\">I2cDevice</a>&lt;'_, M, BUS&gt;<div class=\"where\">where\n    M: <a class=\"trait\" href=\"embassy_sync/blocking_mutex/raw/trait.RawMutex.html\" title=\"trait embassy_sync::blocking_mutex::raw::RawMutex\">RawMutex</a>,\n    BUS: <a class=\"trait\" href=\"embedded_hal/blocking/i2c/trait.Read.html\" title=\"trait embedded_hal::blocking::i2c::Read\">Read</a>&lt;Error = E&gt;,</div>"]]],["embassy_nrf",[["impl&lt;'a, T: <a class=\"trait\" href=\"embassy_nrf/twim/trait.Instance.html\" title=\"trait embassy_nrf::twim::Instance\">Instance</a>&gt; <a class=\"trait\" href=\"embedded_hal/blocking/i2c/trait.Read.html\" title=\"trait embedded_hal::blocking::i2c::Read\">Read</a> for <a class=\"struct\" href=\"embassy_nrf/twim/struct.Twim.html\" title=\"struct embassy_nrf::twim::Twim\">Twim</a>&lt;'a, T&gt;"]]]]);
    if (window.register_implementors) {
        window.register_implementors(implementors);
    } else {
        window.pending_implementors = implementors;
    }
})()
//{"start":57,"fragment_lengths":[744,422]}