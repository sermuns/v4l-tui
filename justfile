watch:
	watchexec --watch src --stop-timeout 0 --restart --wrap-process none --clear -- cargo run 2>/dev/null
