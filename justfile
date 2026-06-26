watch:
	watchexec --watch src --stop-timeout 0 --restart --wrap-process session --clear -- cargo run 2>/dev/null
