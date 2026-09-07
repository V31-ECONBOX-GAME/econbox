BEGIN {
	today = int(now / 86400)
	split("Jan Feb Mar Apr May Jun Jul Aug Sep Oct Nov Dec", MONTH, " ")
}

function civil(epoch_day,   z, era, doe, yoe, doy, mp, year, month, day) {
	z = epoch_day + 719468
	era = int(z / 146097)
	doe = z - era * 146097
	yoe = int((doe - int(doe / 1460) + int(doe / 36524) - int(doe / 146096)) / 365)
	doy = doe - (365 * yoe + int(yoe / 4) - int(yoe / 100))
	mp = int((5 * doy + 2) / 153)
	year = yoe + era * 400
	month = mp + (mp < 10 ? 3 : -9)
	day = doy - int((153 * mp + 2) / 5) + 1
	if (month <= 2) {
		year++
	}
	return day " " MONTH[month] " " year
}

function short_civil(epoch_day,   parts) {
	split(civil(epoch_day), parts, " ")
	return parts[1] " " parts[2]
}

function grouped(number,   point, whole, rest, out) {
	point = index(number, ".")
	whole = (point > 0) ? substr(number, 1, point - 1) : number
	rest = (point > 0) ? substr(number, point) : ""
	while (length(whole) > 3) {
		out = "," substr(whole, length(whole) - 2) out
		whole = substr(whole, 1, length(whole) - 3)
	}
	return whole out rest
}

function hours(minutes,   figure) {
	figure = sprintf("%.1f", minutes / 60)
	sub(/\.0$/, "", figure)
	return figure
}

function plural(count, word) {
	return (count == 1) ? word : word "s"
}

function width_of(text, size) {
	return length(text) * size * 0.52
}

function svg(line) {
	print line > card
}

function markdown(line) {
	print line > block
}

function credit(moment, opened, minutes, day) {
	# Over the gap opens a sitting, credited flat.
	opened = (moments == 0 || moment - previous > gap)
	minutes = opened ? opening : (moment - previous) / 60
	if (opened) {
		sittings++
	}
	total += minutes

	day = today - int(moment / 86400)
	if (day < 0) {
		day = 0
	}

	if (day < recent) {
		recently += minutes
	}

	if (day < days) {
		by_day[day] += minutes
		if (by_day[day] > tallest) {
			tallest = by_day[day]
		}
	}
	if (day + 1 > lived) {
		lived = day + 1
	}

	previous = moment
	moments++
}

{
	if (commits == 0) {
		oldest = $1
	}
	credit($1)
	commits++
}

END {
	if (commits == 0) {
		exit
	}

	# The commit being made is not in the log yet.
	if (now > previous) {
		credit(now)
		commits++
	}

	if (lived < days) {
		days = lived
	}
	if (days < 1) {
		days = 1
	}

	first_seen = civil(int(oldest / 86400))
	last_seen = civil(today)
	last_played = short_civil(today)

	headline = hours(total)
	played = grouped(headline) " " plural(headline, "hour")
	recent_hours = hours(recently)
	summary = grouped(recent_hours) " " plural(recent_hours, "hour") \
			" in the last " recent " " plural(recent, "day") "   \302\267   " \
			grouped(sittings) " " plural(sittings, "sitting") "   \302\267   " \
			grouped(commits) " " plural(commits, "commit")

	PAD = 24
	LABEL_Y = 40
	HEADLINE_Y = 62
	BASE = 134
	AXIS_Y = 152
	CEILING = 46
	STAT_SIZE = 16
	SUMMARY_SIZE = 12
	HEIGHT = 168
	MINIMUM_BAR = 3

	width = int(PAD + width_of(played, STAT_SIZE) + 46 + width_of(summary, SUMMARY_SIZE) + PAD)
	span = width - PAD * 2
	bar_gap = span * 0.32 / days
	if (bar_gap > 6) {
		bar_gap = 6
	}
	bar = int((span - (days - 1) * bar_gap) / days)
	if (bar < 2) {
		bar = 2
	}
	bar_step = (days > 1) ? (span - bar) / (days - 1) : 0

	printf "" > card
	svg("<svg xmlns=\"http://www.w3.org/2000/svg\" width=\"" width "\" height=\"" HEIGHT "\"" \
			" viewBox=\"0 0 " width " " HEIGHT "\" role=\"img\"" \
			" aria-label=\"Play time: " headline " " plural(headline, "hour") \
					" over " commits " " plural(commits, "commit") "," \
					" last played " last_played "\">")
	svg("  <style>")
	svg("    .card { fill: #0d1117; stroke: #30363d }")
	svg("    text { font-family: -apple-system, BlinkMacSystemFont, \"Segoe UI\", Helvetica, Arial, sans-serif }")
	svg("    .label { font-size: 11px; letter-spacing: 1.7px; fill: #8b949e }")
	svg("    .stat { font-size: " STAT_SIZE "px; font-weight: 600; letter-spacing: normal; fill: #e6edf3 }")
	svg("    .note { font-size: " SUMMARY_SIZE "px; fill: #8b949e }")
	svg("    .axis { font-size: 10px; fill: #6e7681 }")
	svg("    .bar { fill: #388bfd }")
	svg("    .bar-quiet { fill: #21262d }")
	svg("  </style>")
	svg("  <rect class=\"card\" x=\"0.5\" y=\"0.5\" width=\"" (width - 1) "\" height=\"" (HEIGHT - 1) "\" rx=\"8\"/>")
	svg("  <text class=\"label\" x=\"" PAD "\" y=\"" LABEL_Y "\">PLAY TIME</text>")
	svg("  <text class=\"stat\" x=\"" PAD "\" y=\"" HEADLINE_Y "\">" played "</text>")
	svg("  <text class=\"label\" x=\"" (width - PAD) "\" y=\"" LABEL_Y "\" text-anchor=\"end\">LAST PLAYED" \
			" <tspan class=\"stat\">" last_played "</tspan></text>")
	svg("  <text class=\"note\" x=\"" (width - PAD) "\" y=\"" HEADLINE_Y "\" text-anchor=\"end\">" summary "</text>")

	for (i = 0; i < days; i++) {
		minutes = by_day[days - 1 - i] + 0
		height = (tallest > 0) ? minutes / tallest * CEILING : 0
		if (height < MINIMUM_BAR) {
			height = MINIMUM_BAR
			style = (minutes > 0) ? "bar" : "bar-quiet"
		}
		else {
			style = "bar"
		}
		svg(sprintf("  <rect class=\"%s\" x=\"%.1f\" y=\"%.1f\" width=\"%d\" height=\"%.1f\" rx=\"1.5\"/>",
				style, PAD + i * bar_step, BASE - height, bar, height))
	}

	svg("  <text class=\"axis\" x=\"" PAD "\" y=\"" AXIS_Y "\">" first_seen "</text>")
	svg("  <text class=\"axis\" x=\"" (width / 2) "\" y=\"" AXIS_Y "\" text-anchor=\"middle\">" \
			grouped(lived) " " plural(lived, "day") "</text>")
	svg("  <text class=\"axis\" x=\"" (width - PAD) "\" y=\"" AXIS_Y "\" text-anchor=\"end\">" last_seen "</text>")
	svg("</svg>")
	close(card)

	printf "" > block
	markdown("")
	markdown("![Play time](.idea/readme/image/time-on-record.svg)")
	markdown("")
	markdown("<details>")
	markdown("<summary>How this is counted</summary>")
	markdown("")
	markdown("Commits record when work was saved, never how long it took, so this is an")
	markdown("estimate rather than a timesheet. Commits less than " int(gap / 60) " " \
			plural(int(gap / 60), "minute") " apart")
	markdown("count as one sitting and contribute the real time between them; a commit that")
	markdown("opens a sitting contributes a flat " opening " " plural(opening, "minute") \
			" for the work that led up to")
	markdown("it. Merges are skipped, and nothing that was never committed is visible here.")
	markdown("")
	markdown("Covers every author. Regenerated on each commit by `.githooks/coding-time`,")
	markdown("which reads commit timestamps and nothing else. `GAP_MINUTES`, `OPENING_MINUTES`,")
	markdown("`RECENT_DAYS` and `DAYS` change what it assumes.")
	markdown("")
	markdown("</details>")
	markdown("")
	close(block)
}
