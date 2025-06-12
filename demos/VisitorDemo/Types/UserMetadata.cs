namespace VisitorDemo;

public class UserMetadata(int version) : IType<User>, IStructType<User>
{
    public void Accept(ITypeVisitor<User> visitor) => visitor.VisitStruct(a => a, this);

    public void Accept(IStructVisitor<User> visitor)
    {
        visitor.VisitField("Name", s => s.Name, StringMetadata.Instance);
        if (version >= 2)
            visitor.VisitField("Age", s => s.Age, IntMetadata.Instance);
        visitor.VisitField("Contacts", s => s.Contacts, new VersionedContactsMetadata(version));
    }
}
