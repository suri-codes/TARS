#import "@preview/cheq:0.2.2": checklist

#show: checklist
#show link: underline
#set heading(numbering: "1.a.1. ")

= TARS (Task Accumulation and Reminder Service)

Really simple to-do app that just takes in notes and stores them globally and then retrieves them when I want to just see them all.

Would be cool to have a tui interface as well.

maybe try to integrate mcp? seems really cool

sqlite for task database

need to really figure out what we want

lowkey gonna just go play warframe gang


= TODO
- [ ] daemon setup for reminding
- [ ] tui setup
- [ ] cli setup


= A reminding daemon
for later btw
https://github.com/hoodie/notify-rust

= command interface
// what the fuck do i do with this
tars add "work on x and y and something else gang wtf are you doing"


entry {
  group:
  prio:
  description:
  due-by:
}

tars list
prints out all tasks in prio order

