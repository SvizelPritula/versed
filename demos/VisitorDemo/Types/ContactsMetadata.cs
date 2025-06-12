namespace VisitorDemo;

public class ContactsMetadata(int version) : IType<Contacts>, IStructType<Contacts>
{
    public void Accept(ITypeVisitor<Contacts> visitor) => visitor.VisitStruct(a => a, this);

    public void Accept(IStructVisitor<Contacts> visitor)
    {
        visitor.VisitField("Email", s => s.Email, StringMetadata.Instance);
        visitor.VisitField("PhoneNumber", s => s.PhoneNumber, StringMetadata.Instance);
    }
}
