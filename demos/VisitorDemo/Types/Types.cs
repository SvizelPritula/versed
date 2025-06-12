namespace VisitorDemo;

public class User
{
    public required string Name;
    public required int Age;
    public required Contacts Contacts;
}

public class Contacts
{
    public required string Email;
    public required string PhoneNumber;
}
