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

foreach (int version in Enumerable.Range(1, 3))
{
    Console.WriteLine($"=== Version {version} ===");
    new UserMetadata(version).Accept(new DebugWriterVisitor<User>(user, Console.Out, 0));
}
