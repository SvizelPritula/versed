namespace VisitorDemo;

public class User
{
    public required string Name { get; set; }
    public required int Age { get; set; }
    public required Contacts Contacts { get; set; }
}

public class Contacts
{
    public required string Email { get; set; }
    public required string PhoneNumber { get; set; }
}
