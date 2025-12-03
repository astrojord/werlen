# werlen

This [Ratatui] app generates stock lists for the magic item shops run by the not-so-humble retired adventurer Werlen in my D&D games. 
He stocks a variety of spell scrolls, low-rarity items, and a special stock of the good stuff for those that pay for his subscription service.

Users are responsible for providing their own shop stock, obtained from WoTC, via the setting `stock_source`. The application expects a CSV file with columns `name`, `price`, `category`, `rarity` for items (leave empty for scrolls), and `level` for scrolls (leave empty for items).

[Ratatui]: https://ratatui.rs

### To do
- [] Finish implementing special stock selector - this can be a scroll or an item
- [] Swap to sqlite instead of reading CSV into memory and holding structs for app runtime - will also allow better querying of items
- [] Use `rand::distributions::WeightedIndex` to properly weight shop stock selection. There should be more low level scrolls and low rarity items than high level/rarity.
- [] ASCII art?