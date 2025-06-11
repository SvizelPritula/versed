namespace VisitorDemo;

public class UserMetadata : IType<User>, IStructType<User>
{
    public void Accept(ITypeVisitor<User> visitor) => visitor.VisitStruct(this);

    public void Accept(IStructVisitor<User> visitor)
    {
        visitor.VisitField("Name", s => s.Name, (s, v) => s.Name = v, StringMetadata.Instance);
        visitor.VisitField("Age", s => s.Age, (s, v) => s.Age = v, IntMetadata.Instance);
        visitor.VisitField("Contacts", s => s.Contacts, (s, v) => s.Contacts = v, new ContactsMetadata());
    }
}
