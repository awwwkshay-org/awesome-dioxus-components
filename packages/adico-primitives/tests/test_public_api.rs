//! Compile-time coverage for the public primitive facade.

use adico_primitives::combobox::{Combobox, ComboboxMulti, ComboboxMultiProps, ComboboxProps};
use adico_primitives::dialog::{
    DialogContent, DialogContentProps, DialogDescription, DialogDescriptionProps, DialogRoot,
    DialogRootProps, DialogTitle, DialogTitleProps,
};
use adico_primitives::select::{
    Select, SelectGroup, SelectGroupLabel, SelectItemIndicator, SelectList, SelectMulti,
    SelectOption, SelectProps, SelectTrigger, SelectValue,
};

#[test]
fn dialog_public_facade_remains_importable() {
    let _ = DialogRoot;
    let _ = DialogContent;
    let _ = DialogTitle;
    let _ = DialogDescription;
    let _ = std::any::type_name::<DialogRootProps>();
    let _ = std::any::type_name::<DialogContentProps>();
    let _ = std::any::type_name::<DialogTitleProps>();
    let _ = std::any::type_name::<DialogDescriptionProps>();
}

#[test]
fn select_public_facade_remains_importable() {
    let _ = Select::<String>;
    let _ = SelectMulti::<String>;
    let _ = SelectGroup;
    let _ = SelectGroupLabel;
    let _ = SelectItemIndicator;
    let _ = SelectList;
    let _ = SelectOption::<String>;
    let _ = SelectTrigger;
    let _ = SelectValue;
    let _ = std::any::type_name::<SelectProps<String>>();
}

#[test]
fn combobox_multi_public_facade_remains_importable() {
    let _ = Combobox::<String>;
    let _ = ComboboxMulti::<String>;
    let _ = std::any::type_name::<ComboboxProps<String>>();
    let _ = std::any::type_name::<ComboboxMultiProps<String>>();
}
