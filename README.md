## Stound

Stound is a small timetable viewer offering an alternative to heavyweight calendar applications.

Specify your timetable in TOML format as per the example below:

    [[category]]
    name = "Work"
    colour = {r = 114, g = 36, b = 108}

    [[category]]
    name = "Sport"
    colour = {r = 4, g = 135, b = 191}

    [[schedule]]
    title = "Client Conference"
    category = "Work"
    time = "12:00"
    length = "1h30m"

        [[schedule.when]]
        day = "Monday"

        [[schedule.when]]
        day = "Thursday"

    [[schedule]]
    title = "Product Direction Meeting"
    category = "Work"

        [[schedule.when]]
        day = "Tuesday"
        time = "11:00"
        length = "1h"

        [[schedule.when]]
        day = "Friday"
        time = "15:00"
        length = "1h20m"

    [[schedule]]
    title = "Rugby Training"
    category = "Sport"

    day = "Friday"
    time = "18:30"
    length = "2h"

    [[schedule]]
    title = "Morning Run"
    category = "Sport"

    time = "8:00"

        [[schedule.when]]
        day = "Monday"
        length = "30m"

        [[schedule.when]]
        day = "Wednesday"
        length = "45m"

Then run the command:

    stound --template ~/path/to/cal.html myschedule.toml

This outputs an HTML document to stdout which when viewed looks like this:

![Example Timetable](https://github.com/jgbyrne/stound/blob/master/example.png)
