#include<time.h>
typedef struct
{
    int part_size;
    int free_space;
    char operatingsystem[20];
    char version[10];
    char date[25];
    char userName[32];
    char password[32];
    
}diskheader;
diskheader in;
char *password(char *a)
{
    char t;
    short i;
    for(i=0;i<32;)
    {
        t=getch();
        if(t==10||t==13||t=='\t')
        {
            a[i]=0;
            printf("\n");
            break;
        }
        if(t==127&&i>=1)
        {
            i--;
            printf("\b");
            printf(" ");
            printf("\b");
        }
        else
        {
            a[i]=t;
            printf("*");
            i++;
        }
        
    }
    return a;
}
int create_raw_partition(char *name)
{
    char a=0;
    FILE *f;
    int i;
    int n;
    clrscr();
    printf("\t\t step 1......\n");
    printf("\nenter the size in Megabytes you want to create a new partition\n(minimum size for pslos should be 1mb)\nEnsure you have minimum space available in your disk to create partition\n:::>");
    scanf("%d",&n);
    f=fopen(name,"w+b");
    if(f==0)
        printf("\nnot enough space in disk to create partition");
    else
    {
        for(i=0;i<n*1024*1024;i++)
            fwrite(&a,sizeof(a),1,f);
        
        fclose(f);
        printf("created a raw partition...press any key to go to step 2..\n");
        return n*1024*1024;
    }
    fclose(f);
}
void psllogo()
{
    printf("\t\t                  (c)\n");
    printf("\t\t ###### ######### \n");
    printf("\t\t #    # #       # \n");
    printf("\t\t ###### ######  # \n");
    printf("\t\t #           #  # \n");
    printf("\t\t #############  ######\n");
    printf("\t\t pyro synctronics unlimited india\n\n");
    printf("\t   The virtual crossplatform operating system\n\n");
}
char *decrypt(char *pass,char *user)
{
    int i;
    for(i=0;i<strlen(pass);i++)
    {
        if(pass[i]==0)
            return pass;
        else
            pass[i]=pass[i]-user[0]-user[1]-user[2];
    }
    return pass;
}
char *encrypt(char *pass,char *user)
{
    int i;
    for(i=0;i<strlen(pass);i++)
    {
        if(pass[i]==0)
            return pass;
        else
            pass[i]=pass[i]+user[0]+user[1]+user[2];
    }
    return pass;
}
void userdata()
{
    char temppass1[32],temppass2[32],user[32];
    int i;
    time_t t;
    time(&t);
    strcpy(in.date,ctime(&t));
    for(i=0;i<25;i++)
        if(in.date[i]==10)
            in.date[i]=0;
    printf("enter user name for the root user(max 32 charachters)::> ");
    gets(user);
    printf("set system password (max 32 charachters)::> ");
    password(temppass1);
    printf("confirm the password ::> ");
    password(temppass2);
    if(strcmp(temppass1,temppass2)==0)
    {
        strcpy(in.userName,user);
        strcpy(in.password,encrypt(temppass1,user));
        
        printf("\n\nmove to step 3....\n\n");
    }
    else
    {
        clrscr();
        printf("type mismatch!!!\n");
        userdata();
    }
}
void install(char *path)
{
    FILE *fp;
    
    
    printf("\n\ninstalling.......\n\n");
    fp=fopen(path,"r+");
    fwrite(&in,sizeof(in),1,fp);
    fclose(fp);
    printf("%d \n %d \n %s \n %s \n %s \n %s \n %s ",in.part_size,in.free_space,in.operatingsystem,in.version,in.date,in.userName,in.password);
    printf("created a new partition");
    getch();
}

void install_pslos(char *path)
{
    clrscr();
    psllogo();
    printf("This operating systems comes out without any warrenty \nyou can enhance use and redistribute this operating system\nif you install this operating system you are welcomed to open source world...\nI hope you will enjoy this os\nand will help in its further development\nthanx... you can send your ideas and views at everlasting.puneet@gmail.com\npress any key to continue installation else close the window........");
    getch();
    in.part_size=create_raw_partition(path);
    in.free_space=in.part_size-sizeof(in);
    strcpy(in.operatingsystem,"PSLOS virtual");
    strcpy(in.version,"0.0.1");	
    getch();
    getch();
    clrscr();
    printf("\t\t step 2......\n");
    userdata();
    getch();
    clrscr();
    printf("\t\t step 3......\n");
    printf("prepare the partition and finally install PSLOS\npress 'y'to continue 'n' to stop here");
    if(getch()=='y')	
        install(path);
    else 
        exit(1);
}



