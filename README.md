# werlen

This [Ratatui] app generates stock lists for the magic item shops run by the not-so-humble retired adventurer Werlen in my D&D games. 
He stocks a variety of spell scrolls, low-rarity items, and a special stock of the good stuff for those that pay for his subscription service.

Users are responsible for providing their own shop stock, obtained from WoTC, via the command line argument `stock_source`. The application expects a CSV file with columns `name`, `price`, `rarity` for items (leave empty for scrolls), and `level` for scrolls (leave empty for items).

[Ratatui]: https://ratatui.rs
