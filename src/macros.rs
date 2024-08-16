#[macro_export]
macro_rules! define_handle {
    ( $name:ident ) => {
        struct $name;

        impl poise::serenity_prelude::prelude::TypeMapKey for $name {
            type Value = tokio::task::JoinHandle<()>;
        }
    };
}

// #[macro_export]
// macro_rules! embed {
//     (
//         $( $title:expr  ;   )?
//         $( =>$desc:expr ;   )?
//         $( ($field_name:expr, $field_value:expr $(, $inline:expr )? ) ),* $(,)?
//     ) => {
//         embed! {
//             $( $title; )?
//             $( =>$desc ;   )?
//             $( ($field_name, $field_value $(, $inline )?) )*
//             : $crate::DEFAULT_COLOR
//         }
//     };

//     (
//         $( $title:expr  ;   )?
//         $( =>$desc:expr ;   )?
//         $( ($field_name:expr, $field_value:expr $(, $inline:expr )? ) ),* $(,)?
//         red
//     ) => {
//         embed! {
//             $( $title; )?
//             $( =>$desc ;   )?
//             $( ($field_name, $field_value $(, $inline )?) )*
//             : poise::serenity_prelude::Color::RED
//         }
//     };

//     (
//         $( $title:expr  ;   )?
//         $( =>$desc:expr ;   )?
//         $( ($field_name:expr, $field_value:expr $(, $inline:expr )? ) ),* $(,)?
//         :$color:expr
//     ) => {
//         poise::serenity_prelude::CreateEmbed::new()
//             $( .title($title) )?
//             $( .description($desc) )?
//             $(
//                 .field(
//                     $field_name,
//                     $field_value,
//                     embed!(@field $($inline)?)
//                 )
//             )*
//             .color($color)
//     };

//     (
//         @field $inline:expr
//     ) => {
//         $inline
//     };

//     (
//         @field
//     ) => {
//         false
//     };
// }

// #[test]
// fn test_embed_macro() {
//     let embed = embed! {
//         "Hi";
//         =>"Bro";
//         ( "name", "value" )
//         :0x2
//     };

//     assert_eq!(
//         embed,
//         poise::serenity_prelude::CreateEmbed::new()
//             .title("Hi")
//             .description("Bro")
//             .field("name", "value", false)
//             .color(0x2)
//     )
// }
