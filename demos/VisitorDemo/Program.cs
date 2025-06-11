using VisitorDemo;

User user = new User
{
    Name = "Peter Parker",
    Age = 28,
    Contacts = new Contacts
    {
        Email = "peter.parker@example.org",
        PhoneNumber = "+1 (311) 555-2368"
    }
};

new UserMetadata().Accept(new DebugWriterVisitor<User>(user, Console.Out, 0));
