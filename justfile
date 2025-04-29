id := "st.lynx.plugins.opendeck-akp153.sdPlugin"

package: build zip

build:
    cargo build --release
    rm -r build
    mkdir -p build/{{id}}
    cp -r assets build/{{id}}
    cp manifest.json build/{{id}}
    cp target/release/opendeck-akp153 build/{{id}}

[working-directory: "build"]
zip:
    zip -r opendeck-akp153.sdPlugin {{id}}/
