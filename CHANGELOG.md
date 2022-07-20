# Changelog

## LolRofl 0.3.0

This release adds a lot of documentation examples and tests as well as some breaking changes when manipulating payload data

### Added

* Added a new `model::section::PacketTime` enum to represent payload packet time
* Added a new `model::section::GenericSection::time` method to get the timestamp of a payload packet
* Added a new `model::section::GenericSection::params` method to get the parameters of a packet
* Added `--start-time` and `--end-time` filter to `lolrofl analyze` command-line

### Changed

* Changed the signature of `Rofl::segment_iter` and `iter::PayloadIterator::new` to accept an additional boolean parameter to parse payload data (`true`) or only headers (`false`)

### Fixed

* Fixed `GenericSection::from_slice` to ensure it would work on arbitrary packets

## LolRofl 0.2.0

This release features a lot of cleanup in the public API as well as huge design changes in the way data is loaded and stored. This should be treated as a completely different version from 0.1.0.

A notable naming convention change is the replacement of the `segment data` label by `section`. This change is reflected in structs names.

### Added

* Added a `Rofl::segment_iter` method to iterate over the segments of a file
* Added a `iter::PayloadIterator` construct to iterate over the segments of a payload

### Changed

* Moved most structural classes from the crate's root to the `model` subscope.
* Moved `SegmentIterator` to `iter::SegmentIterator`. The `iter` subscope will contain all future iterators
* Moved `error::Errors` to the crate's root (`Errors`)
* Moved and renamed `segments::SegmentDataCore` to `model::section::SectionCore`
* Moved and renamed `segments::GenericDataSegment` to `model::section::GenericSection`
* Many changes to the public interface of `Rofl`:
  * Dropped ALL `load_*` methods. Getters are now the only way to access an object
  * Renamed `meta` to `metadata`. Changed the method's return value to a `Result<&str, Errors>`
  * Changed the `payload` function's return value to a `Result<Payload, Errors>`
  * Changed `from_slice`'s signature to only need a slice (dropped the `config` parameter)
  * Removed the `chunks` and `keyframes` functions. The replacement for those is `segment_iter`
* Removed the `consts` subscope as the values it held were not of use anymore
* Removed the `segments::StartSegment` construct as it could not be used. Future versions should bring back an equivalent

### Fixed

* The `payload` feature may now be disabled without issues. Disabling it will make it impossible to load segment data within `iter::PayloadIterator`

## LolRofl 0.1.0

Initial unstable release
