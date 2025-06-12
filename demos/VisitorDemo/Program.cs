using VisitorDemo;

foreach (int version in Enumerable.Range(1, 3))
{
    User user = new()
    {
        Name = "Peter Parker",
        Age = 28,
        Contacts = new()
        {
            Email = "peter.parker@example.org",
            PhoneNumber = "+1 (311) 555-2368"
        }
    };
    var metadata = new UserMetadata(version);

    Console.WriteLine($"=== Version {version} ===");

    metadata.Accept(new DebugWriterVisitor<User>(user, Console.Out, 0));

    metadata.Accept(new TrademakerVisitor<User>(ref user));

    Console.WriteLine($"== Trademarked ==");

    metadata.Accept(new DebugWriterVisitor<User>(user, Console.Out, 0));
}
