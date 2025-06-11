namespace VisitorDemo;

public class ContactsMetadata : IType<Contacts>, IStructType<Contacts>
{
    public void Accept(ITypeVisitor<Contacts> visitor) => visitor.VisitStruct(this);

    public void Accept(IStructVisitor<Contacts> visitor)
    {
        visitor.VisitField("Email", s => s.Email, (s, v) => s.Email = v, StringMetadata.Instance);
        visitor.VisitField("PhoneNumber", s => s.PhoneNumber, (s, v) => s.PhoneNumber = v, StringMetadata.Instance);
    }
}
