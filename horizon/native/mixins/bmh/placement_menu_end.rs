fn bmh_menu_end(value: Result<(), &'static str>) -> Result<(), &'static str> {
    bmh_placement::menu_end();
    value
}
